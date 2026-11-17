use super::*;
use gpui::{Modifiers, TestAppContext};

#[gpui::test]
fn code_surface_stays_in_column_across_tab_switches_and_reflow(cx: &mut TestAppContext) {
    let source = format!(
        "before\n\n```rust\nfn main() {{ println!(\"hello\"); }}\n{}\n```\n\nafter",
        "x".repeat(500)
    );
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::VisualEdit;
        app.sidebar_visible = false;
        app.outline_visible = false;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    let cached = app.read_with(cx, |app, _| {
        app.active_tab().document.visual_blocks_shared()
    });
    let version = app.read_with(cx, |app, _| app.active_tab().document.version());
    let index = cached
        .iter()
        .position(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
        .unwrap();
    let surface: &'static str = Box::leak(format!("visual-code-surface-{index}").into_boxed_str());
    let content: &'static str = Box::leak(format!("visual-block-content-{index}").into_boxed_str());
    for width in [1182., 740., 1500., 1182.] {
        cx.simulate_resize(size(px(width), px(800.)));
        for transition in 0..6 {
            if transition % 2 == 0 {
                cx.dispatch_action(NewTab);
                cx.run_until_parked();
                cx.dispatch_action(CloseTab);
            } else {
                app.update(cx, |app, cx| {
                    app.open_in_new_tab(
                        MarkdownDocument::from_text("other\n\n```\nshort\n```"),
                        cx,
                    );
                    app.switch_active_tab(0, cx);
                });
            }
            app.update(cx, |app, cx| app.switch_active_tab(0, cx));
            cx.run_until_parked();
            let code = cx.debug_bounds(surface).unwrap();
            let column = cx.debug_bounds(content).unwrap();
            assert!(
                code.left() >= column.left() - px(0.5) && code.right() <= column.right() + px(0.5),
                "width={width}, transition={transition}, code={code:?}, column={column:?}"
            );
            app.update(cx, |app, _| {
                assert_eq!(app.active_tab().document.text(), source);
                assert!(app.active_tab().undo_stack.is_empty());
                assert_eq!(app.active_tab().document.version(), version);
            });
        }
    }
}

#[gpui::test]
fn image_hover_selection_and_old_expansion_never_reveal_raw_payload(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.simulate_resize(size(px(1180.), px(800.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for stable in [false, true] {
        for source in [
            "![图](missing.png \"Caption\")".to_string(),
            "![图][asset]\n\n[asset]: missing.png \"Caption\"".to_string(),
            format!("![图](data:image/png;base64,{})", "QUJD".repeat(32768)),
        ] {
            let (version, cache, id) = app.update(cx, |app, cx| {
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                app.active_tab = 0;
                app.view_mode = ViewMode::VisualEdit;
                app.stable_visual_inline = stable;
                app.sidebar_visible = false;
                let blocks = app.active_tab().document.visual_blocks_shared();
                let block = blocks
                    .iter()
                    .find(|block| matches!(block.kind, VisualBlockKind::Image { .. }))
                    .unwrap();
                let id = block.id;
                app.active_tab_mut().toggle_visual_source_expanded(id);
                app.active_tab_mut().hovered_visual_source_block = Some(id);
                app.active_tab_mut().visual_last_projection = None;
                app.visual_field_projection_builds.set(0);
                cx.notify();
                (app.active_tab().document.version(), blocks, id)
            });
            cx.run_until_parked();
            let row = cx.debug_bounds("visual-block-row-0").unwrap();
            cx.simulate_mouse_move(row.center(), None, Modifiers::none());
            cx.run_until_parked();
            app.update(cx, |app, cx| app.move_to(1, cx));
            cx.run_until_parked();
            app.update(cx, |app, cx| {
                assert_eq!(app.active_tab().document.text(), source);
                assert_eq!(app.active_tab().document.version(), version);
                assert!(app.active_tab().undo_stack.is_empty());
                assert!(Arc::ptr_eq(
                    &cache,
                    &app.active_tab().document.visual_blocks_shared()
                ));
                assert_eq!(
                    app.visual_field_projection_builds.get(),
                    0,
                    "stable={stable}, id={id:?}"
                );
                assert!(app.active_tab().visual_last_projection.is_none());
                assert_visual_image_object_selected(app);
                let target = BlockTarget::from_block(version, &cache[0]);
                app.open_visual_edit_context_menu(target, point(px(100.), px(100.)), cx);
                let items = app.block_menu.as_ref().unwrap().root_items();
                assert!(items.contains(&BlockMenuItem::ImageProperties));
                for duplicate in [
                    BlockMenuItem::CopyBlock,
                    BlockMenuItem::CutBlock,
                    BlockMenuItem::DeleteSelection,
                ] {
                    assert!(
                        !items.contains(&duplicate),
                        "whole-image selection must not duplicate {duplicate:?}"
                    );
                }
                assert!(!items.iter().any(|item| matches!(
                    item,
                    BlockMenuItem::Transform(_) | BlockMenuItem::Insert(_)
                )));
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("visual-image-properties").is_some());
            assert!(cx.debug_bounds("visual-image-edit-source").is_none());
        }
    }
}

#[gpui::test]
fn image_properties_preserve_exact_occurrence_metadata_cancel_stale_and_undo(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for eol in ["\n", "\r\n"] {
        let source = format!(
            "before{eol}{eol}![图](old.png \"Cap {{width=50 align=right}}\"){eol}{eol}![other](old.png){eol}{eol}after"
        );
        let start = source.find("![图]").unwrap();
        app.update_in(cx, |app, window, cx| {
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
            app.active_tab = 0;
            app.view_mode = ViewMode::VisualEdit;
            app.active_tab_mut().selected_range = 1..3;
            let version = app.active_tab().document.version();
            let cache = app.active_tab().document.visual_blocks_shared();
            app.open_image_properties_at(start, cx);
            let editor = app.link_editor.as_ref().unwrap();
            assert_eq!(editor.kind, LinkEditorKind::ImageProperties);
            assert_eq!(editor.label, "图");
            assert_eq!(editor.title, "Cap");
            assert_eq!(editor.url, "old.png");
            app.clear_file_tree_search(&ClearFileTreeSearch, window, cx);
            assert!(app.link_editor.is_none());
            assert_eq!(app.active_tab().selected_range, 1..3);
            app.open_image_properties_at(start, cx);
            app.confirm_link_editor(window, cx);
            assert_eq!(app.active_tab().document.text(), source);
            assert_eq!(app.active_tab().document.version(), version);
            assert!(app.active_tab().undo_stack.is_empty());
            assert!(Arc::ptr_eq(
                &cache,
                &app.active_tab().document.visual_blocks_shared()
            ));
            app.open_image_properties_at(start, cx);
            app.link_editor.as_mut().unwrap().url = "new.png".into();
            app.confirm_link_editor(window, cx);
            assert_eq!(
                app.active_tab().document.text(),
                source.replacen("old.png", "new.png", 1)
            );
            assert_eq!(app.active_tab().undo_stack.len(), 1);
            assert_visual_image_object_selected(app);
            app.undo(&Undo, window, cx);
            assert_eq!(app.active_tab().document.text(), source);
            assert_eq!(app.active_tab().selected_range, 1..3);
            app.open_image_properties_at(start, cx);
            let editor = app.link_editor.as_mut().unwrap();
            editor.label = "新图🙂".into();
            editor.title = "新说明".into();
            app.confirm_link_editor(window, cx);
            assert_eq!(
                app.active_tab().document.text(),
                source.replace(
                    "![图](old.png \"Cap {width=50 align=right}\")",
                    "![新图🙂](old.png \"新说明 {width=50 align=right}\")"
                )
            );
            app.undo(&Undo, window, cx);
            assert_eq!(app.active_tab().document.text(), source);
            app.open_image_properties_at(start, cx);
            app.link_editor.as_mut().unwrap().url = "stale.png".into();
            let mutation = app
                .active_tab()
                .document
                .prepare_whole_mutation(MutationOrigin::ExternalReload, "external change");
            app.apply_document_mutation("image_properties_stale_test", mutation);
            app.confirm_link_editor(window, cx);
            assert_eq!(app.active_tab().document.text(), "external change");
            assert!(app.active_tab().undo_stack.is_empty());
        });
        let source = format!(
            "![原图][shared]{eol}{eol}![other][shared]{eol}{eol}[shared]: old.png \"Cap\"{eol}"
        );
        app.update_in(cx, |app, window, cx| {
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
            app.active_tab = 0;
            app.view_mode = ViewMode::VisualEdit;
            app.open_image_properties_at(0, cx);
            app.confirm_link_editor(window, cx);
            assert_eq!(app.active_tab().document.text(), source);
            app.open_image_properties_at(0, cx);
            app.link_editor.as_mut().unwrap().url = "new.png".into();
            app.confirm_link_editor(window, cx);
            assert_eq!(
                app.active_tab().document.text(),
                source.replacen("![原图][shared]", "![原图](new.png \"Cap\")", 1)
            );
            assert_eq!(app.active_tab().undo_stack.len(), 1);
            app.undo(&Undo, window, cx);
            assert_eq!(app.active_tab().document.text(), source);
        });
    }
}

#[gpui::test]
fn image_object_selection_typing_and_enter_never_edit_hidden_markdown(cx: &mut TestAppContext) {
    let source = "before\n\n![图](missing.png)\n\nafter";
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
    app.update_in(cx, |app, window, cx| {
        app.move_to(source.find("missing").unwrap(), cx);
        assert_visual_image_object_selected(app);
        app.insert_newline(&InsertNewline, window, cx);
        assert!(
            app.active_tab()
                .document
                .text()
                .contains("![图](missing.png)")
        );
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        app.undo(&Undo, window, cx);
        assert_eq!(app.active_tab().document.text(), source);
        app.move_to(source.find("missing").unwrap(), cx);
        app.replace_text_in_range(None, "替换图片", window, cx);
        assert!(!app.active_tab().document.text().contains("missing.png"));
        assert!(app.active_tab().document.text().contains("替换图片"));
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        app.undo(&Undo, window, cx);
        assert_eq!(app.active_tab().document.text(), source);
    });
}

fn assert_visual_image_object_selected(app: &MarkionApp) {
    let range = app.active_tab().selected_range.clone();
    assert!(!range.is_empty());
    assert!(app.active_tab().document.text()[range].starts_with("!["));
}

const LIFECYCLE_TABLE_SOURCE: &str = "before\n\n| A | B | C |\n| --- | --- | --- |\n| alpha | beta | gamma |\n| delta | epsilon | zeta |\n\nafter";

fn empty_context_target(app: &MarkionApp, offset: usize) -> BlockTarget {
    let doc = &app.active_tab().document;
    let blocks = doc.visual_blocks_shared();
    let block = blocks
        .iter()
        .find(|block| {
            markion::block_edit::empty_paragraph_range(doc.text(), block, offset).is_some()
        })
        .unwrap();
    BlockTarget::from_block(doc.version(), block)
}

#[gpui::test]
fn context_empty_paragraph_insert_actions_settings_and_undo(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for ending in ["\n", "\r\n"] {
        for (raw, original_offset) in [
            ("", 0),
            ("before\n", 7),
            ("before\n\nnext", 7),
            ("before\n\n\n\nnext", 8),
            (" \t", 1),
        ] {
            let source = raw.replace('\n', ending);
            let offset = raw[..original_offset].replace('\n', ending).len();
            let actions = app.update(cx, |app, cx| {
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                app.active_tab = 0;
                app.view_mode = ViewMode::VisualEdit;
                app.active_tab_mut().selected_range = offset..offset;
                app.open_visual_edit_context_menu(
                    empty_context_target(app, offset),
                    point(px(50.), px(50.)),
                    cx,
                );
                let presentation = app.block_menu_presentation().unwrap();
                assert!(presentation.is_empty_paragraph);
                assert!(presentation.can_insert_block);
                for item in [
                    BlockMenuItem::CopyBlock,
                    BlockMenuItem::CutBlock,
                    BlockMenuItem::Delete,
                    BlockMenuItem::CopySelection,
                ] {
                    assert!(!presentation.item_enabled(item));
                }
                let actions = app
                    .block_menu
                    .as_ref()
                    .unwrap()
                    .submenu_items(BlockContextSubmenu::Insertion);
                assert_eq!(actions.len(), 21);
                assert!(
                    actions.iter().all(|item| presentation.item_enabled(*item)),
                    "{source:?}: {presentation:?}"
                );
                actions
            });
            for action in actions {
                app.update_in(cx, |app, window, cx| {
                    app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                    app.active_tab_mut().selected_range = offset..offset;
                    app.open_visual_edit_context_menu(
                        empty_context_target(app, offset),
                        point(px(50.), px(50.)),
                        cx,
                    );
                    let cache = app.active_tab().document.visual_blocks_shared();
                    let version = app.active_tab().document.version();
                    activate_context_item(app, action, Some(window), cx);
                    if matches!(
                        action,
                        BlockMenuItem::Insert(
                            SlashPaletteCommand::Link
                                | SlashPaletteCommand::Formula
                                | SlashPaletteCommand::Image
                        )
                    ) {
                        let range = app
                            .link_editor
                            .as_ref()
                            .map(|editor| editor.source_range.clone())
                            .or_else(|| {
                                app.token_editor
                                    .as_ref()
                                    .map(|editor| editor.source_range.clone())
                            })
                            .unwrap();
                        assert_eq!(range, offset..offset);
                        assert_eq!(app.active_tab().document.text(), source);
                        assert_eq!(app.active_tab().document.version(), version);
                        assert!(Arc::ptr_eq(
                            &cache,
                            &app.active_tab().document.visual_blocks_shared()
                        ));
                        app.cancel_link_editor(cx);
                        app.cancel_token_editor(cx);
                        assert!(app.active_tab().undo_stack.is_empty());
                    } else {
                        assert!(app.block_menu.is_none());
                        assert_eq!(
                            app.active_tab().undo_stack.len(),
                            1,
                            "{source:?}: {action:?}"
                        );
                        assert!(app.active_tab().selected_range.is_empty());
                        let inserted = app.active_tab().document.text().to_string();
                        app.undo(&Undo, window, cx);
                        assert_eq!(app.active_tab().document.text(), source);
                        assert_eq!(app.active_tab().selected_range, offset..offset);
                        app.redo(&Redo, window, cx);
                        assert_eq!(app.active_tab().document.text(), inserted);
                        app.undo(&Undo, window, cx);
                    }
                });
            }
        }
    }
}

#[gpui::test]
fn context_empty_paragraph_pointer_reuses_clicked_line_with_other_selection(
    cx: &mut TestAppContext,
) {
    let source = "before\n\n\n\nnext";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.view_mode = ViewMode::VisualEdit;
        app.sidebar_visible = false;
        app.outline_visible = false;
        app
    });
    cx.simulate_resize(size(px(1180.), px(800.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let (index, range, metrics) = app.read_with(cx, |app, _| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Whitespace))
            .unwrap();
        (
            index,
            blocks[index].source_range.clone(),
            app.typography_metrics(),
        )
    });
    let row = cx
        .debug_bounds(Box::leak(
            format!("visual-block-row-{index}").into_boxed_str(),
        ))
        .unwrap();
    let offset = whitespace_source_at_line(range.clone(), 1, source);
    let position = point(
        row.left() + px(30.),
        row.top()
            + px(whitespace_line_top(
                range,
                1,
                source,
                metrics.paragraph_line_height,
                metrics.paragraph_spacing,
            ) + metrics.paragraph_line_height * 0.5),
    );
    for selection in [0..3, 0..0] {
        app.update(cx, |app, cx| {
            app.active_tab_mut().selected_range = selection.clone();
            cx.notify();
        });
        cx.run_until_parked();
        let current_row = cx
            .debug_bounds(Box::leak(
                format!("visual-block-row-{index}").into_boxed_str(),
            ))
            .unwrap();
        let position = point(
            current_row.left() + px(30.),
            current_row.top() + position.y - row.top(),
        );
        cx.simulate_event(MouseUpEvent {
            button: MouseButton::Right,
            position,
            modifiers: Modifiers::none(),
            click_count: 1,
        });
        cx.run_until_parked();
        app.update_in(cx, |app, window, cx| {
            let menu = app.block_menu.as_ref().unwrap();
            assert_eq!(
                menu.empty_paragraph_offset,
                Some(offset),
                "selection={selection:?}, row={current_row:?}, pointer={position:?}, list={:?}",
                app.active_tab().visual_list.bounds_for_item(index)
            );
            assert_eq!(app.active_tab().selected_range, selection);
            if selection.is_empty() {
                assert_eq!(menu.insertion_range, Some(offset..offset));
            }
            activate_context_item(
                app,
                BlockMenuItem::Insert(SlashPaletteCommand::Block(SlashCommand::Heading(2))),
                Some(window),
                cx,
            );
            assert_eq!(
                app.active_tab().document.text(),
                format!("{}## {}", &source[..offset], &source[offset..])
            );
            app.undo(&Undo, window, cx);
            assert_eq!(app.active_tab().document.text(), source);
            assert_eq!(app.active_tab().selected_range, selection);
        });
    }
}

#[gpui::test]
fn context_new_callout_body_and_title_clicks_type_in_body_and_undo(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.simulate_resize(size(px(1180.), px(800.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for ending in ["\n", "\r\n"] {
        for kind in [
            AlertKind::Note,
            AlertKind::Tip,
            AlertKind::Important,
            AlertKind::Warning,
            AlertKind::Caution,
        ] {
            for empty in [false, true] {
                let source = if empty {
                    "before\n\n\nafter"
                } else {
                    "before\n\nanchor\n\nafter"
                }
                .replace('\n', ending);
                let blank_offset = "before\n".replace('\n', ending).len();
                app.update_in(cx, |app, window, cx| {
                    app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                    app.active_tab = 0;
                    app.view_mode = ViewMode::VisualEdit;
                    app.sidebar_visible = false;
                    app.outline_visible = false;
                    app.active_tab_mut().selected_range = if empty {
                        blank_offset..blank_offset
                    } else {
                        0..0
                    };
                    let target = if empty {
                        empty_context_target(app, blank_offset)
                    } else {
                        prose_target(app, "anchor")
                    };
                    app.open_visual_edit_context_menu(target, point(px(80.), px(80.)), cx);
                    activate_context_item(
                        app,
                        BlockMenuItem::InsertCallout(kind),
                        Some(window),
                        cx,
                    );
                    assert_eq!(app.active_tab().undo_stack.len(), 1);
                });
                cx.run_until_parked();
                let (title_index, body_index, body_offset, inserted) =
                    app.read_with(cx, |app, _| {
                        let tab = app.active_tab();
                        let blocks = tab.document.visual_blocks_shared();
                        let title = blocks
                            .iter()
                            .position(|block| {
                                matches!(block.kind, VisualBlockKind::CalloutTitle { .. })
                            })
                            .unwrap();
                        let body = &blocks[title + 1];
                        assert!(matches!(body.kind, VisualBlockKind::Whitespace));
                        let offset = whitespace_navigation_offset(
                            body.source_range.clone(),
                            tab.document.text(),
                            true,
                        );
                        assert_eq!(tab.selected_range, offset..offset);
                        (title, title + 1, offset, tab.document.text().to_string())
                    });
                for title_click in [false, true] {
                    // Move away first: a no-op click must not pass merely
                    // because insertion/Undo already left the caret in body.
                    app.update(cx, |app, cx| app.move_to(0, cx));
                    cx.run_until_parked();
                    let (cache, version) = app.read_with(cx, |app, _| {
                        (
                            app.active_tab().document.visual_blocks_shared(),
                            app.active_tab().document.version(),
                        )
                    });
                    let selector = if title_click {
                        format!("visual-callout-title-{title_index}")
                    } else {
                        format!("visual-block-content-{body_index}")
                    };
                    let bounds = cx
                        .debug_bounds(Box::leak(selector.into_boxed_str()))
                        .unwrap();
                    if !title_click {
                        assert!(bounds.size.height >= px(20.));
                    }
                    cx.simulate_click(bounds.center(), Modifiers::none());
                    cx.run_until_parked();
                    app.update(cx, |app, _| {
                        assert_eq!(app.active_tab().selected_range, body_offset..body_offset);
                        assert_eq!(app.active_tab().document.text(), inserted);
                        assert_eq!(app.active_tab().document.version(), version);
                        assert!(Arc::ptr_eq(
                            &cache,
                            &app.active_tab().document.visual_blocks_shared()
                        ));
                    });
                    cx.simulate_input("正文中文🙂");
                    cx.run_until_parked();
                    app.update_in(cx, |app, window, cx| {
                        let expected = format!(
                            "{}正文中文🙂{}",
                            &inserted[..body_offset],
                            &inserted[body_offset..]
                        );
                        assert_eq!(app.active_tab().document.text(), expected);
                        assert!(expected.contains(&format!(
                            "> [!{}]{ending}> 正文中文🙂",
                            kind.label().to_uppercase()
                        )));
                        app.undo(&Undo, window, cx);
                        assert_eq!(app.active_tab().document.text(), inserted);
                        app.redo(&Redo, window, cx);
                        assert_eq!(app.active_tab().document.text(), expected);
                        app.undo(&Undo, window, cx);
                    });
                    cx.run_until_parked();
                }
                app.update_in(cx, |app, window, cx| {
                    app.undo(&Undo, window, cx);
                    assert_eq!(app.active_tab().document.text(), source);
                });
            }
        }
    }
}

#[gpui::test]
fn context_top_level_block_controls_and_bottom_insertion_are_reachable(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("one\n\ntwo"))];
        app.view_mode = ViewMode::VisualEdit;
        app.sidebar_visible = false;
        app.outline_visible = false;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for (width, height, theme) in [(1180., 800., AppTheme::Paper), (340., 280., AppTheme::Ink)] {
        cx.simulate_resize(size(px(width), px(height)));
        let cache = app.update(cx, |app, cx| {
            app.theme = theme;
            app.active_tab_mut().selected_range = 0..2;
            app.open_visual_edit_context_menu(
                prose_target(app, "two"),
                point(px(width - 10.), px(height - 10.)),
                cx,
            );
            let menu = app.block_menu.as_ref().unwrap();
            let root = menu.root_items();
            assert_eq!(root[0], BlockMenuItem::Transform(BlockTransform::Text));
            let clipboard = root
                .iter()
                .position(|item| *item == BlockMenuItem::CutSelection)
                .unwrap();
            for item in [
                BlockMenuItem::Transform(BlockTransform::Heading(5)),
                BlockMenuItem::Transform(BlockTransform::TaskList),
                BlockMenuItem::Duplicate,
                BlockMenuItem::Delete,
            ] {
                assert!(
                    root.iter()
                        .position(|candidate| *candidate == item)
                        .unwrap()
                        < clipboard
                );
            }
            assert_eq!(
                root.last(),
                Some(&BlockMenuItem::Submenu(BlockContextSubmenu::Insertion))
            );
            assert_eq!(
                root.iter()
                    .filter(|item| matches!(item, BlockMenuItem::Submenu(_)))
                    .count(),
                1
            );
            let items = menu.submenu_items(BlockContextSubmenu::Insertion);
            assert!(!root.contains(&BlockMenuItem::Transform(BlockTransform::Heading(6))));
            assert!(
                !items.contains(&BlockMenuItem::Insert(SlashPaletteCommand::Block(
                    SlashCommand::Heading(6)
                )))
            );
            assert_eq!(items.len(), 21);
            assert!(
                !items
                    .iter()
                    .any(|item| matches!(item, BlockMenuItem::Transform(_)))
            );
            app.active_tab().document.visual_blocks_shared()
        });
        cx.run_until_parked();
        if width > 600. {
            let transform = cx
                .debug_bounds(block_menu_item_element_id(BlockMenuItem::Transform(
                    BlockTransform::Text,
                )))
                .unwrap();
            let clipboard = cx.debug_bounds("visual-selection-cut").unwrap();
            assert!(transform.top() < clipboard.top());
            assert!(cx.debug_bounds("visual-context-block-format").is_none());
        }
        app.update(cx, |app, cx| {
            app.open_visual_context_submenu(BlockContextSubmenu::Insertion, true, cx);
        });
        cx.run_until_parked();
        for _ in 0..21 {
            cx.dispatch_action(Down);
            cx.run_until_parked();
            let item = app.read_with(cx, |app, _| {
                let menu = app.block_menu.as_ref().unwrap();
                menu.visible_items()[menu.selected_index()]
            });
            let button = cx.debug_bounds(block_menu_item_element_id(item)).unwrap();
            let child = cx.debug_bounds("visual-context-flyout").unwrap();
            assert!(
                child.contains(&button.center()),
                "{item:?}: {button:?} outside {child:?}"
            );
        }
        app.update(cx, |app, _| {
            assert_eq!(app.active_tab().selected_range, 0..2);
            assert_eq!(app.active_tab().document.text(), "one\n\ntwo");
            assert!(app.active_tab().undo_stack.is_empty());
            assert!(Arc::ptr_eq(
                &cache,
                &app.active_tab().document.visual_blocks_shared()
            ));
        });
        cx.dispatch_action(ClearFileTreeSearch);
        cx.run_until_parked();
    }
}

#[gpui::test]
fn context_insert_blocks_preserve_clicked_content_selection_and_atomic_undo(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    let mut actions: Vec<_> = SlashCommand::ALL
        .into_iter()
        .filter(|command| *command != SlashCommand::Heading(6))
        .map(|command| BlockMenuItem::Insert(SlashPaletteCommand::Block(command)))
        .collect();
    actions.extend(
        [
            AlertKind::Note,
            AlertKind::Tip,
            AlertKind::Important,
            AlertKind::Warning,
            AlertKind::Caution,
        ]
        .map(BlockMenuItem::InsertCallout),
    );
    for ending in ["\n", "\r\n"] {
        for action in actions.iter().copied() {
            let source = "before 中文🙂\n\nclicked\n\nafter".replace('\n', ending);
            app.update_in(cx, |app, window, cx| {
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                app.active_tab = 0;
                app.view_mode = ViewMode::VisualEdit;
                app.active_tab_mut().selected_range = 0..6;
                app.open_visual_edit_context_menu(prose_target(app, "clicked"), point(px(50.), px(50.)), cx);
                activate_context_item(app, action, Some(window), cx);
                assert!(app.block_menu.is_none());
                let text = app.active_tab().document.text();
                assert!(text.starts_with(&format!("before 中文🙂{ending}{ending}clicked{ending}{ending}")), "{action:?}: {text:?}");
                assert!(text.ends_with("after"));
                assert_eq!(app.active_tab().undo_stack.len(), 1, "{action:?}");
                assert!(app.active_tab().selected_range.is_empty());
                if let BlockMenuItem::InsertCallout(kind) = action {
                    assert!(text.contains(&format!("> [!{}]{ending}> ", kind.label().to_uppercase())));
                    assert!(app.active_tab().document.visual_blocks_shared().iter().any(|block| matches!(block.kind, VisualBlockKind::CalloutTitle { kind: actual, .. } if actual == kind)));
                }
                let inserted = text.to_string();
                if action != BlockMenuItem::Insert(SlashPaletteCommand::Block(SlashCommand::Divider)) {
                    let caret = app.active_tab().selected_range.start;
                    let mut typed = inserted.clone();
                    typed.insert_str(caret, "新内容🙂");
                    app.replace_text_in_range(None, "新内容🙂", window, cx);
                    assert_eq!(app.active_tab().document.text(), typed, "typing after {action:?}");
                    app.undo(&Undo, window, cx);
                    assert_eq!(app.active_tab().document.text(), inserted);
                }
                app.undo(&Undo, window, cx);
                assert_eq!(app.active_tab().document.text(), source);
                assert_eq!(app.active_tab().selected_range, 0..6);
                app.redo(&Redo, window, cx);
                assert_eq!(app.active_tab().document.text(), inserted);
                app.undo(&Undo, window, cx);
                app.open_visual_edit_context_menu(prose_target(app, "clicked"), point(px(50.), px(50.)), cx);
                open_context_submenu_for(app, action, cx);
                app.active_tab_mut().selected_range = 1..1;
                app.activate_visual_block_menu_item(action, Some(window), cx);
                assert_eq!(app.active_tab().document.text(), source);
                assert!(app.active_tab().undo_stack.is_empty());
            });
        }
    }
}

#[gpui::test]
fn table_context_hover_flyout_preserves_root_targets_and_viewport(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            LIFECYCLE_TABLE_SOURCE,
        ))];
        app.view_mode = ViewMode::VisualEdit;
        app.sidebar_visible = false;
        app.outline_visible = false;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for (width, height, theme) in [
        (1180., 800., AppTheme::Paper),
        (1180., 800., AppTheme::Ink),
        (340., 280., AppTheme::Paper),
        (340., 280., AppTheme::Ink),
    ] {
        for page in [
            BlockContextSubmenu::TableRows,
            BlockContextSubmenu::TableColumns,
        ] {
            cx.simulate_resize(size(px(width), px(height)));
            let (version, cached, selection) = app.update(cx, |app, cx| {
                app.theme = theme;
                app.active_tab_mut().selected_range = 0..3;
                app.open_visual_edit_context_menu(
                    lifecycle_table_target(app),
                    point(px(width - 10.), px(height - 10.)),
                    cx,
                );
                let menu = app.block_menu.as_ref().unwrap();
                assert!(menu.table_context.is_none());
                assert!(!menu.root_items().iter().any(|item| matches!(
                    item,
                    BlockMenuItem::Transform(_) | BlockMenuItem::Insert(_)
                )));
                let start = LIFECYCLE_TABLE_SOURCE.find("alpha").unwrap();
                app.active_tab_mut().selected_range = start..start + 3;
                app.open_visual_pointer_context_menu(
                    lifecycle_table_target(app),
                    point(px(width / 2.), px(100.)),
                    Some((1, 0)),
                    cx,
                );
                let menu = app.block_menu.as_ref().unwrap();
                let items = menu.root_items();
                assert_eq!(
                    items
                        .iter()
                        .filter(|item| matches!(item, BlockMenuItem::Submenu(_)))
                        .count(),
                    2
                );
                assert!(items.contains(&BlockMenuItem::Submenu(BlockContextSubmenu::TableRows)));
                assert!(items.contains(&BlockMenuItem::Submenu(BlockContextSubmenu::TableColumns)));
                assert!(items.contains(&BlockMenuItem::Delete));
                assert!(!items.iter().any(|item| matches!(
                    item,
                    BlockMenuItem::Transform(_)
                        | BlockMenuItem::Insert(_)
                        | BlockMenuItem::TableRow(_)
                        | BlockMenuItem::TableColumn(_)
                )));
                assert_eq!(
                    items
                        .iter()
                        .filter(|item| **item
                            == BlockMenuItem::SelectionFormat(SelectionFormatAction::Link))
                        .count(),
                    1
                );
                (
                    app.active_tab().document.version(),
                    app.active_tab().document.visual_blocks_shared(),
                    start..start + 3,
                )
            });
            cx.run_until_parked();
            let root = cx.debug_bounds("visual-block-menu-panel").unwrap();
            let parent = cx
                .debug_bounds(block_menu_item_element_id(BlockMenuItem::Submenu(page)))
                .unwrap();
            cx.simulate_mouse_move(parent.center(), None, Modifiers::none());
            cx.run_until_parked();
            assert_eq!(
                root,
                cx.debug_bounds("visual-block-menu-panel").unwrap(),
                "hover must not move the root"
            );
            let child = cx.debug_bounds("visual-context-flyout").unwrap();
            assert!(
                child.left() >= px(0.)
                    && child.right() <= px(width)
                    && child.top() >= px(0.)
                    && child.bottom() <= px(height),
                "{child:?}"
            );
            if width > 600. {
                assert!(
                    child.left() >= root.right() - px(2.) || child.right() <= root.left() + px(2.),
                    "root={root:?}, child={child:?}"
                );
            }
            app.update(cx, |app, _| {
                let menu = app.block_menu.as_ref().unwrap();
                assert_eq!(menu.submenu, Some(page));
                assert!(menu.submenu_items(page).iter().all(|item| {
                    if page == BlockContextSubmenu::TableRows {
                        matches!(item, BlockMenuItem::TableRow(_))
                    } else {
                        matches!(item, BlockMenuItem::TableColumn(_))
                    }
                }));
                if width > 600. {
                    assert!(!menu.submenu_focused);
                }
            });
            cx.dispatch_action(Right);
            cx.run_until_parked();
            let count = app.read_with(cx, |app, _| {
                app.block_menu.as_ref().unwrap().visible_items().len()
            });
            assert_eq!(
                count,
                if page == BlockContextSubmenu::TableRows {
                    7
                } else {
                    10
                }
            );
            for _ in 0..count {
                cx.dispatch_action(Down);
                cx.run_until_parked();
                let item = app.read_with(cx, |app, _| {
                    let menu = app.block_menu.as_ref().unwrap();
                    menu.visible_items()[menu.selected_index()]
                });
                let button = cx.debug_bounds(block_menu_item_element_id(item)).unwrap();
                let child = cx.debug_bounds("visual-context-flyout").unwrap();
                assert!(
                    child.contains(&button.center()),
                    "{item:?}: {button:?} outside {child:?}"
                );
            }
            app.update(cx, |app, cx| {
                let menu = app.block_menu.as_ref().unwrap();
                let selected = menu.submenu_selected;
                app.select_visual_block_menu_root(3, Some(BlockContextSubmenu::Insertion), cx);
                assert_eq!(
                    app.block_menu.as_ref().unwrap().submenu_selected,
                    selected,
                    "old flyout events must be rejected"
                );
                assert_eq!(app.active_tab().document.text(), LIFECYCLE_TABLE_SOURCE);
                assert_eq!(app.active_tab().document.version(), version);
                assert_eq!(app.active_tab().selected_range, selection);
                assert!(app.active_tab().undo_stack.is_empty());
                assert!(Arc::ptr_eq(
                    &cached,
                    &app.active_tab().document.visual_blocks_shared()
                ));
            });
            cx.dispatch_action(Left);
            cx.run_until_parked();
            app.update(cx, |app, _| {
                let menu = app.block_menu.as_ref().unwrap();
                assert!(
                    menu.submenu.is_none(),
                    "page={:?}, keyboard={}, pointer={:?}, suppressed={:?}",
                    menu.submenu,
                    menu.keyboard_navigation,
                    menu.pointer_position,
                    menu.suppressed_hover
                );
                assert!(menu.geometry.borrow().child.is_none());
            });
            cx.dispatch_action(ClearFileTreeSearch);
            cx.run_until_parked();
        }
    }
    // A real click in the side flyout must not be dismissed by root click-away.
    cx.simulate_resize(size(px(1180.), px(800.)));
    app.update(cx, |app, cx| {
        app.open_visual_pointer_context_menu(
            lifecycle_table_target(app),
            point(px(100.), px(100.)),
            Some((1, 0)),
            cx,
        )
    });
    cx.run_until_parked();
    let parent = cx
        .debug_bounds("visual-context-table-rows")
        .unwrap()
        .center();
    cx.simulate_mouse_move(parent, None, Modifiers::none());
    cx.run_until_parked();
    let button = cx.debug_bounds("visual-context-row-insert-after").unwrap();
    cx.simulate_click(button.center(), Modifiers::none());
    cx.run_until_parked();
    app.update_in(cx, |app, window, cx| {
        assert!(app.block_menu.is_none());
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        app.undo(&Undo, window, cx);
        assert_eq!(app.active_tab().document.text(), LIFECYCLE_TABLE_SOURCE);
    });
}

#[gpui::test]
fn table_shift_enter_inserts_rendered_cell_break_and_preserves_exact_bytes(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.simulate_resize(size(px(1182.), px(800.)));
    cx.update(|window, cx| {
        cx.clear_key_bindings();
        bind_app_keys(cx, &BTreeMap::new());
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for eol in ["\n", "\r\n"] {
        let source = format!(
            "before{eol}{eol}| 标题中文🙂尾 | B |{eol}| :--- | ---: |{eol}| 内容中文🙂尾 | keep |{eol}{eol}after"
        );
        for needle in ["标题中文🙂尾", "内容中文🙂尾"] {
            let start = source.find(needle).unwrap();
            let emoji = start + "标题中文".len();
            for range in [
                emoji..emoji,
                emoji..emoji + "🙂".len(),
                start + needle.len()..start + needle.len(),
            ] {
                app.update(cx, |app, cx| {
                    app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                    app.active_tab = 0;
                    app.view_mode = ViewMode::VisualEdit;
                    app.sidebar_visible = false;
                    app.outline_visible = false;
                    app.active_tab_mut().selected_range = range.clone();
                    cx.notify();
                });
                cx.run_until_parked();
                cx.simulate_keystrokes("shift-enter");
                cx.run_until_parked();
                cx.dispatch_action(Left);
                cx.run_until_parked();
                app.update(cx, |app, _| {
                    assert_eq!(app.active_tab().selected_range, range.start..range.start)
                });
                cx.dispatch_action(Right);
                cx.run_until_parked();
                let mut expected = source.clone();
                expected.replace_range(range.clone(), "<br>");
                app.update_in(cx, |app, window, cx| {
                    assert_eq!(app.active_tab().document.text(), expected);
                    let caret = range.start + 4;
                    assert_eq!(app.active_tab().selected_range, caret..caret);
                    assert_eq!(app.active_tab().undo_stack.len(), 1);
                    let rendered = &app.active_tab().visual_last_projection.as_ref().unwrap().0;
                    assert!(
                        rendered.contains('\n') && !rendered.contains("<br>"),
                        "{rendered:?}"
                    );
                    let field = app
                        .active_tab()
                        .document
                        .visual_editor_field_at(&(caret..caret))
                        .unwrap();
                    assert!(matches!(
                        field.kind,
                        VisualEditorFieldKind::TableCell { .. }
                    ));
                    app.replace_text_in_range(None, "新🙂", window, cx);
                    assert!(app.active_tab().document.text().contains("<br>新🙂"));
                    app.undo(&Undo, window, cx);
                    assert_eq!(app.active_tab().document.text(), expected);
                    app.undo(&Undo, window, cx);
                    assert_eq!(app.active_tab().document.text(), source);
                    assert_eq!(app.active_tab().selected_range, range);
                    app.redo(&Redo, window, cx);
                    app.backspace(&Backspace, window, cx);
                    let mut joined = source.clone();
                    joined.replace_range(range.clone(), "");
                    assert_eq!(app.active_tab().document.text(), joined);
                    app.undo(&Undo, window, cx);
                    app.active_tab_mut().selected_range = range.start..range.start;
                    app.delete(&Delete, window, cx);
                    assert_eq!(app.active_tab().document.text(), joined);
                });
            }
        }
    }
}

#[test]
fn table_break_projection_distinguishes_html_from_inline_code_and_escaped_tags() {
    let source = "| 中文<br>🙂<br/>c<BR />d \\<br> `<br>` | B |\n| --- | --- |\n| x | y |";
    let document = MarkdownDocument::from_text(source);
    let blocks = document.visual_blocks_shared();
    let VisualBlockEditor::Table { cells } = blocks
        .iter()
        .find_map(|block| block.editor.as_ref())
        .unwrap()
    else {
        panic!("table");
    };
    let cell = &cells[0];
    assert_eq!(cell.hard_breaks.len(), 3);
    let (projection, _) = visual_table_editing_projection(source, &cell.field, &cell.hard_breaks);
    assert_eq!(projection.text, "中文\n🙂\nc\nd \\<br> `<br>`");
    for range in &cell.hard_breaks {
        let display = projection.display_for_source(range.start).unwrap();
        assert_eq!(projection.source_for_display(display), range.start);
        assert_eq!(projection.source_for_display(display + 1), range.end);
        assert_eq!(
            document
                .visual_atomic_token_edit(range.end, false)
                .unwrap()
                .range,
            *range
        );
        assert_eq!(
            document
                .visual_atomic_token_edit(range.start, true)
                .unwrap()
                .range,
            *range
        );
    }
}

fn open_context_submenu_for(
    app: &mut MarkionApp,
    item: BlockMenuItem,
    cx: &mut Context<MarkionApp>,
) {
    let page = app.block_menu.as_ref().and_then(|menu| {
        if menu.root_items().contains(&item) {
            return None;
        }
        [
            BlockContextSubmenu::TableRows,
            BlockContextSubmenu::TableColumns,
            BlockContextSubmenu::Insertion,
            BlockContextSubmenu::ImageScale,
        ]
        .into_iter()
        .find(|page| {
            menu.root_items().contains(&BlockMenuItem::Submenu(*page))
                && menu.submenu_items(*page).contains(&item)
        })
    });
    if let Some(page) = page {
        app.open_visual_context_submenu(page, true, cx);
    }
}

fn activate_context_item(
    app: &mut MarkionApp,
    item: BlockMenuItem,
    window: Option<&mut Window>,
    cx: &mut Context<MarkionApp>,
) {
    open_context_submenu_for(app, item, cx);
    app.activate_visual_block_menu_item(item, window, cx);
}

fn lifecycle_table_target(app: &MarkionApp) -> BlockTarget {
    let doc = &app.active_tab().document;
    let blocks = doc.visual_blocks_shared();
    let block = blocks
        .iter()
        .find(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
        .unwrap();
    BlockTarget::from_block(doc.version(), block)
}

#[gpui::test]
fn table_menu_lifecycle_outside_cell_text_padding_and_same_cell_clicks(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            LIFECYCLE_TABLE_SOURCE,
        ))];
        app.view_mode = ViewMode::VisualEdit;
        app.sidebar_visible = false;
        app
    });
    cx.simulate_resize(size(px(1180.), px(800.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let index = app.read_with(cx, |app, _| {
        app.active_tab()
            .document
            .visual_blocks_shared()
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
            .unwrap()
    });
    let origin_id: &'static str =
        Box::leak(format!("visual-table-cell-{index}-1-0").into_boxed_str());
    let other_id: &'static str =
        Box::leak(format!("visual-table-cell-{index}-2-2").into_boxed_str());
    for axis_menu in [false, true] {
        for (same, padding) in [(false, false), (false, true), (true, true), (true, false)] {
            let origin = cx.debug_bounds(origin_id).unwrap();
            let anchor = point(origin.left() + px(10.), origin.center().y);
            let (version, blocks) = app.update(cx, |app, cx| {
                let target = lifecycle_table_target(app);
                if axis_menu {
                    let blocks = app.active_tab().document.visual_blocks_shared();
                    let block = blocks
                        .iter()
                        .find(|block| block.id == target.block_id)
                        .unwrap();
                    let axis = VisualTableAxisTarget::for_block(
                        app.active_tab().document.version(),
                        block,
                        VisualTableAxis::Row(1),
                    )
                    .unwrap();
                    app.open_visual_table_menu(axis, anchor, cx);
                } else {
                    app.open_visual_pointer_context_menu(target, anchor, Some((1, 0)), cx);
                }
                (
                    app.active_tab().document.version(),
                    app.active_tab().document.visual_blocks_shared(),
                )
            });
            cx.run_until_parked();
            let bounds = cx
                .debug_bounds(if same { origin_id } else { other_id })
                .unwrap();
            let point = point(
                bounds.left() + px(if padding { 2. } else { 10. }),
                bounds.center().y,
            );
            cx.simulate_click(point, Modifiers::none());
            cx.run_until_parked();
            app.update(cx, |app, _| {
                assert!(
                    app.block_menu.is_none() && app.visual_table_menu.is_none(),
                    "axis={axis_menu}, same={same}, padding={padding}"
                );
                assert_eq!(app.active_tab().document.text(), LIFECYCLE_TABLE_SOURCE);
                assert_eq!(app.active_tab().document.version(), version);
                assert!(app.active_tab().undo_stack.is_empty());
                assert!(Arc::ptr_eq(
                    &blocks,
                    &app.active_tab().document.visual_blocks_shared()
                ));
                let cursor = app.active_tab().cursor_offset();
                let expected = if same { "alpha" } else { "zeta" };
                let start = LIFECYCLE_TABLE_SOURCE.find(expected).unwrap();
                assert!(start <= cursor && cursor <= start + expected.len());
            });
        }
    }
}

#[gpui::test]
fn table_menu_lifecycle_new_tab_shortcut_preserves_original_document(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            LIFECYCLE_TABLE_SOURCE,
        ))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        cx.clear_key_bindings();
        bind_app_keys(cx, &BTreeMap::new());
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        let target = lifecycle_table_target(app);
        app.open_visual_pointer_context_menu(target, point(px(300.), px(240.)), Some((1, 1)), cx);
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("visual-block-menu-panel").is_some());
    cx.simulate_keystrokes("ctrl-shift-n");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab, 1);
        assert_eq!(app.active_tab().document.text(), "");
        assert_eq!(app.tabs[0].document.text(), LIFECYCLE_TABLE_SOURCE);
        assert!(app.tabs[0].undo_stack.is_empty());
        assert!(app.block_menu.is_none() && app.visual_table_menu.is_none());
    });
    assert!(cx.debug_bounds("visual-block-menu-panel").is_none());
}

#[gpui::test]
fn table_menu_lifecycle_document_transitions_clear_bound_overlays(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for transition in 0..7 {
        app.update_in(cx, |app, window, cx| {
            app.tabs = vec![
                EditorTab::new(MarkdownDocument::from_text(LIFECYCLE_TABLE_SOURCE)),
                EditorTab::new(MarkdownDocument::from_text("other document")),
            ];
            app.active_tab = 0;
            app.view_mode = ViewMode::VisualEdit;
            let target = lifecycle_table_target(app);
            app.open_visual_pointer_context_menu(
                target,
                point(px(300.), px(240.)),
                Some((1, 1)),
                cx,
            );
            let stale = app.block_menu.clone().unwrap();
            match transition {
                0 => app.new_tab(&NewTab, window, cx),
                1 => app.replace_active_tab(MarkdownDocument::from_text("replacement"), cx),
                2 => app.switch_active_tab(1, cx),
                3 => app.close_tab_confirmed(cx),
                4 => app.open_image_in_new_tab(PathBuf::from("missing-lifecycle.png"), cx),
                5 => app.replace_active_tab_with_image(PathBuf::from("missing-lifecycle.png"), cx),
                _ => {
                    let target = TabContextTarget::capture(0, &app.tabs[0]);
                    app.remove_tabs_by_identity(&[target], cx);
                }
            }
            assert!(
                app.block_menu.is_none() && app.visual_table_menu.is_none(),
                "transition {transition}"
            );
            assert!(app.editor_context_menu.is_none() && app.preview_context_menu.is_none());
            // Even a late queued callback cannot carry this menu into the new document.
            app.block_menu = Some(stale);
            cx.notify();
        });
        cx.run_until_parked();
        app.update(cx, |app, _| {
            assert!(app.block_menu.is_none() && app.visual_table_menu.is_none());
        });
        assert!(cx.debug_bounds("visual-block-menu-panel").is_none());
    }
}

#[gpui::test]
fn table_menu_lifecycle_stale_render_is_inert_for_document_and_image_tabs(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for image_tab in [false, true] {
        app.update(cx, |app, cx| {
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
                LIFECYCLE_TABLE_SOURCE,
            ))];
            app.active_tab = 0;
            app.view_mode = ViewMode::VisualEdit;
            let target = lifecycle_table_target(app);
            app.open_visual_pointer_context_menu(
                target,
                point(px(300.), px(240.)),
                Some((1, 1)),
                cx,
            );
            let mut menu = app.block_menu.clone().unwrap();
            menu.target.document_version += 1;
            let axis = menu.table_context.unwrap().row;
            app.block_menu = Some(menu);
            app.visual_table_menu = Some(VisualTableMenuState {
                target: axis,
                anchor: point(px(30.), px(40.)),
                alignment: TableAlignment::Default,
            });
            if image_tab {
                app.tabs[0] = EditorTab::new_image(
                    PathBuf::from("missing-lifecycle.png"),
                    PreviewImageKey::from_local_path(&PathBuf::from("missing-lifecycle.png")),
                );
            } else {
                app.visual_table_menu
                    .as_mut()
                    .unwrap()
                    .target
                    .document_version += 1;
            }
            cx.notify();
        });
        cx.run_until_parked();
        app.update(cx, |app, _| {
            assert!(app.block_menu.is_none() && app.visual_table_menu.is_none());
            if !image_tab {
                assert_eq!(app.active_tab().document.text(), LIFECYCLE_TABLE_SOURCE);
                assert!(app.active_tab().undo_stack.is_empty());
            }
        });
    }
}

#[gpui::test]
fn editing_context_clipboard_and_delete_use_exact_selection(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for ending in ["\n", "\r\n"] {
        let source = format!("alpha 中文🙂 omega{ending}{ending}second line{ending}{ending}after");
        let start = source.find("中文").unwrap();
        let partial = start..start + "中文🙂".len();
        let cross_block = start..source.find(" line").unwrap();
        for selection in [partial, cross_block] {
            for action in [
                BlockMenuItem::CopySelection,
                BlockMenuItem::CutSelection,
                BlockMenuItem::DeleteSelection,
                BlockMenuItem::Paste,
            ] {
                app.update_in(cx, |app, window, cx| {
                    app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                    app.view_mode = ViewMode::VisualEdit;
                    app.active_tab_mut().selected_range = selection.clone();
                    let target = prose_target(app, "after");
                    let blocks = app.active_tab().document.visual_blocks_shared();
                    let version = app.active_tab().document.version();
                    app.open_visual_edit_context_menu(target, point(px(40.), px(50.)), cx);
                    let menu = app.block_menu.as_ref().unwrap();
                    for required in [
                        BlockMenuItem::CopyBlock,
                        BlockMenuItem::CutBlock,
                        BlockMenuItem::CopySelection,
                        BlockMenuItem::CutSelection,
                    ] {
                        assert!(menu.root_items().contains(&required));
                    }
                    assert_eq!(app.active_tab().selected_range, selection);
                    assert_eq!(app.active_tab().document.version(), version);
                    assert!(Arc::ptr_eq(
                        &blocks,
                        &app.active_tab().document.visual_blocks_shared()
                    ));
                    app.file_tree_query_focused = true;
                    app.file_tree_query = "unrelated input".into();
                    cx.write_to_clipboard(ClipboardItem::new_string("replacement".into()));
                    activate_context_item(app, action, Some(window), cx);
                    let copied = cx.read_from_clipboard().unwrap().text().unwrap();
                    if matches!(
                        action,
                        BlockMenuItem::CopySelection | BlockMenuItem::CutSelection
                    ) {
                        assert_eq!(copied, source[selection.clone()]);
                    } else {
                        assert_eq!(copied, "replacement");
                    }
                    assert_eq!(app.file_tree_query, "unrelated input");
                    assert!(app.file_tree_query_focused);
                    if action == BlockMenuItem::CopySelection {
                        assert_eq!(app.active_tab().document.text(), source);
                        assert_eq!(app.active_tab().document.version(), version);
                        assert!(app.active_tab().undo_stack.is_empty());
                        assert!(Arc::ptr_eq(
                            &blocks,
                            &app.active_tab().document.visual_blocks_shared()
                        ));
                    } else {
                        let replacement = if action == BlockMenuItem::Paste {
                            "replacement"
                        } else {
                            ""
                        };
                        assert_eq!(
                            app.active_tab().document.text(),
                            format!(
                                "{}{replacement}{}",
                                &source[..selection.start],
                                &source[selection.end..]
                            )
                        );
                        assert_eq!(app.active_tab().undo_stack.len(), 1);
                        app.undo(&Undo, window, cx);
                        assert_eq!(app.active_tab().document.text(), source);
                        assert_eq!(app.active_tab().selected_range, selection);
                    }
                    app.file_tree_query_focused = false;
                });
            }
        }
    }
}

#[gpui::test]
fn editing_context_caret_paste_and_stale_capture_never_fall_back_to_block(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update_in(cx, |app, window, cx| {
        let source = "first\n\nsecond";
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.view_mode = ViewMode::VisualEdit;
        app.active_tab_mut().selected_range = 2..2;
        let target = prose_target(app, "second");
        app.open_visual_edit_context_menu(target, point(px(40.), px(50.)), cx);
        for action in [
            BlockMenuItem::CopySelection,
            BlockMenuItem::CutSelection,
            BlockMenuItem::DeleteSelection,
        ] {
            assert!(!app.block_menu_presentation().unwrap().item_enabled(action));
            activate_context_item(app, action, Some(window), cx);
            assert_eq!(app.active_tab().document.text(), source);
        }
        cx.write_to_clipboard(ClipboardItem::new_string("X".into()));
        activate_context_item(app, BlockMenuItem::Paste, Some(window), cx);
        assert_eq!(app.active_tab().document.text(), "fiXrst\n\nsecond");
        app.undo(&Undo, window, cx);
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().selected_range, 2..2);
        let target = prose_target(app, "second");
        app.open_visual_edit_context_menu(target, point(px(40.), px(50.)), cx);
        activate_context_item(app, BlockMenuItem::SelectAll, Some(window), cx);
        assert_eq!(app.active_tab().selected_range, 0..5);
        assert_eq!(app.active_tab().document.text(), source);
        assert!(app.active_tab().undo_stack.is_empty());
    });
    for stale_kind in 0..3 {
        for action in [
            BlockMenuItem::CopySelection,
            BlockMenuItem::CutSelection,
            BlockMenuItem::DeleteSelection,
            BlockMenuItem::Paste,
        ] {
            app.update_in(cx, |app, window, cx| {
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
                    "first\n\nsecond",
                ))];
                app.active_tab_mut().selected_range = 1..3;
                let target = prose_target(app, "second");
                app.open_visual_edit_context_menu(target, point(px(40.), px(50.)), cx);
                match stale_kind {
                    0 => app.active_tab_mut().selected_range = 2..4,
                    1 => {
                        let mutation = app.active_tab().document.prepare_whole_mutation(
                            MutationOrigin::ExternalReload,
                            "changed source",
                        );
                        assert!(
                            app.apply_document_mutation("context_stale_source", mutation)
                                .is_some()
                        );
                    }
                    _ => {
                        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
                            "first\n\nsecond",
                        ))];
                        app.active_tab_mut().selected_range = 1..3;
                    }
                }
                let text = app.active_tab().document.text().to_string();
                cx.write_to_clipboard(ClipboardItem::new_string("unchanged".into()));
                activate_context_item(app, action, Some(window), cx);
                assert!(app.block_menu.is_none());
                assert_eq!(app.active_tab().document.text(), text);
                assert_eq!(
                    cx.read_from_clipboard().unwrap().text().as_deref(),
                    Some("unchanged")
                );
                assert!(app.active_tab().undo_stack.is_empty());
            });
        }
    }
}

#[gpui::test]
fn combined_context_keeps_clicked_block_commands_and_selection_formats(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for (action, expected) in [
        (
            BlockMenuItem::Transform(BlockTransform::Heading(2)),
            "first 中文\r\n\r\n## clicked\r\n\r\nafter",
        ),
        (
            BlockMenuItem::Transform(BlockTransform::NumberedList),
            "first 中文\r\n\r\n1. clicked\r\n\r\nafter",
        ),
        (
            BlockMenuItem::MoveUp,
            "clicked\r\n\r\nfirst 中文\r\n\r\nafter",
        ),
        (
            BlockMenuItem::MoveDown,
            "first 中文\r\n\r\nafter\r\n\r\nclicked\r\n\r\n",
        ),
        (BlockMenuItem::Delete, "first 中文\r\n\r\nafter"),
    ] {
        app.update_in(cx, |app, window, cx| {
            let source = "first 中文\r\n\r\nclicked\r\n\r\nafter";
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
            app.view_mode = ViewMode::VisualEdit;
            app.active_tab_mut().selected_range = 0..5;
            let target = prose_target(app, "clicked");
            app.open_visual_edit_context_menu(target, point(px(30.), px(40.)), cx);
            let menu = app.block_menu.as_ref().unwrap();
            let mut items = menu.root_items();
            for page in [BlockContextSubmenu::Insertion] {
                items.extend(menu.submenu_items(page));
            }
            for required in [
                BlockMenuItem::Transform(BlockTransform::Text),
                BlockMenuItem::Transform(BlockTransform::Heading(5)),
                BlockMenuItem::Transform(BlockTransform::BulletedList),
                BlockMenuItem::Transform(BlockTransform::CodeBlock),
                BlockMenuItem::Transform(BlockTransform::Divider),
                BlockMenuItem::Transform(BlockTransform::Table),
                BlockMenuItem::Insert(SlashPaletteCommand::Formula),
                BlockMenuItem::MoveUp,
                BlockMenuItem::MoveDown,
                BlockMenuItem::Delete,
                BlockMenuItem::CopySelection,
                BlockMenuItem::CutSelection,
                BlockMenuItem::Paste,
                BlockMenuItem::SelectionFormat(SelectionFormatAction::Strikethrough),
            ] {
                assert!(items.contains(&required), "missing {required:?}");
            }
            activate_context_item(app, action, Some(window), cx);
            assert_eq!(app.active_tab().document.text(), expected, "{action:?}");
            assert_eq!(app.active_tab().undo_stack.len(), 1);
            app.undo(&Undo, window, cx);
            assert_eq!(app.active_tab().document.text(), source);
            assert_eq!(app.active_tab().selected_range, 0..5);
        });
    }
    for (action, marker) in [
        (SelectionFormatAction::Bold, "**"),
        (SelectionFormatAction::Italic, "*"),
        (SelectionFormatAction::InlineCode, "`"),
        (SelectionFormatAction::Strikethrough, "~~"),
    ] {
        for source in [
            "before 中文🙂 after\n\nclicked",
            "| A | B |\n| --- | --- |\n| 中文🙂 | keep |\n\nclicked",
        ] {
            app.update_in(cx, |app, window, cx| {
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
                let start = source.find("中文").unwrap();
                let selected = start..start + "中文🙂".len();
                app.active_tab_mut().selected_range = selected.clone();
                let target = prose_target(app, "clicked");
                app.open_visual_edit_context_menu(target, point(px(30.), px(40.)), cx);
                activate_context_item(
                    app,
                    BlockMenuItem::SelectionFormat(action),
                    Some(window),
                    cx,
                );
                assert_eq!(
                    app.active_tab().document.text(),
                    source.replace("中文🙂", &format!("{marker}中文🙂{marker}"))
                );
                assert_eq!(app.active_tab().undo_stack.len(), 1);
                app.undo(&Undo, window, cx);
                assert_eq!(app.active_tab().document.text(), source);
                assert_eq!(app.active_tab().selected_range, selected);
            });
        }
    }
}

#[gpui::test]
fn combined_context_pointer_paste_and_formats_use_painted_location(cx: &mut TestAppContext) {
    let source = "first\n\nsecond 中文🙂 target\n\nafter";
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
    let offset = source.find("中文").unwrap();
    let position = app.read_with(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|block| block.source_range.contains(&offset))
            .unwrap();
        let snapshot = &tab.visual_navigation_snapshots[&index];
        snapshot
            .lines
            .iter()
            .find_map(|line| {
                line.windows.iter().find_map(|window| {
                    let display = window.projection.display_for_source(offset)?;
                    let position = window.layout.position_for_index(display)?;
                    Some(point(
                        position.x + px(0.1),
                        line.y + window.y_offset + window.layout.line_height() * 0.5,
                    ))
                })
            })
            .unwrap()
    });
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position,
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update_in(cx, |app, window, cx| {
        let menu = app.block_menu.as_ref().unwrap();
        assert_eq!(
            menu.editing_context.as_ref().unwrap().edit_range,
            Some(offset..offset)
        );
        assert_eq!(app.active_tab().selected_range, 1..1);
        assert!(
            menu.root_items()
                .contains(&BlockMenuItem::Transform(BlockTransform::Heading(1)))
        );
        cx.write_to_clipboard(ClipboardItem::new_string("inserted".into()));
        activate_context_item(app, BlockMenuItem::Paste, Some(window), cx);
        assert_eq!(
            app.active_tab().document.text(),
            source.replacen("中文", "inserted中文", 1)
        );
        app.undo(&Undo, window, cx);
        assert_eq!(app.active_tab().document.text(), source);
        app.active_tab_mut().selected_range = 1..1;
    });
    cx.run_until_parked();
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position,
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update_in(cx, |app, window, cx| {
        assert_eq!(
            app.block_menu
                .as_ref()
                .unwrap()
                .selection_format
                .as_ref()
                .unwrap()
                .range,
            offset..offset
        );
        activate_context_item(
            app,
            BlockMenuItem::SelectionFormat(SelectionFormatAction::Strikethrough),
            Some(window),
            cx,
        );
        assert_eq!(
            app.active_tab().document.text(),
            source.replacen("中文", "~~text~~中文", 1)
        );
        app.undo(&Undo, window, cx);
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().selected_range, 1..1);
    });
}

#[gpui::test]
fn combined_context_table_actions_target_clicked_cell_and_preserve_other_table(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    let before = "before\r\n\r\n| untouched | table |\r\n| --- | --- |\r\n| old | bytes |\r\n\r\n";
    let table = "| A | B | C |\r\n| :--- | :---: | ---: |\r\n| a1 | b1 | c1 |\r\n| a2 | b2 | c2 |\r\n| a3 | b3 | c3 |";
    let source = format!("{before}{table}\r\n\r\nafter");
    for column in [false, true] {
        let axis = if column {
            VisualTableAxis::Column(1)
        } else {
            VisualTableAxis::Row(2)
        };
        for action in visual_table_menu_items(axis) {
            app.update_in(cx, |app, window, cx| {
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                app.view_mode = ViewMode::VisualEdit;
                app.active_tab_mut().selected_range = 0..6;
                let blocks = app.active_tab().document.visual_blocks_shared();
                let block = blocks
                    .iter()
                    .filter(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
                    .nth(1)
                    .unwrap();
                let target = BlockTarget::from_block(app.active_tab().document.version(), block);
                let VisualBlockEditor::Table { cells } = block.editor.as_ref().unwrap() else {
                    unreachable!()
                };
                let table_range = app
                    .active_tab()
                    .document
                    .table_range_at(cells[0].field.source_range.start)
                    .unwrap();
                let original_suffix = &source[table_range.end..];
                let cached = blocks.clone();
                app.open_visual_pointer_context_menu(
                    target,
                    point(px(30.), px(40.)),
                    Some((2, 1)),
                    cx,
                );
                let context = app.block_menu.as_ref().unwrap().table_context.unwrap();
                assert_eq!(context.row.axis, VisualTableAxis::Row(2));
                assert_eq!(context.column.axis, VisualTableAxis::Column(1));
                assert_eq!(app.active_tab().selected_range, 0..6);
                assert!(Arc::ptr_eq(
                    &cached,
                    &app.active_tab().document.visual_blocks_shared()
                ));
                let item = if column {
                    BlockMenuItem::TableColumn(*action)
                } else {
                    BlockMenuItem::TableRow(*action)
                };
                assert!(app.block_menu_presentation().unwrap().item_enabled(item));
                open_context_submenu_for(app, item, cx);
                activate_context_item(app, item, Some(window), cx);
                let text = app.active_tab().document.text();
                assert!(
                    text.starts_with(before),
                    "unrelated table changed for {item:?}"
                );
                assert!(
                    text.ends_with(original_suffix),
                    "unrelated suffix changed for {item:?}: {text:?}"
                );
                assert_eq!(app.active_tab().undo_stack.len(), 1, "{item:?}");
                let blocks = app.active_tab().document.visual_blocks_shared();
                let block = blocks
                    .iter()
                    .filter(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
                    .nth(1)
                    .unwrap();
                let VisualBlockKind::Table { rows, alignments } = &block.kind else {
                    unreachable!()
                };
                let grid: Vec<Vec<&str>> = rows
                    .iter()
                    .map(|row| row.iter().map(|cell| cell.text.as_str()).collect())
                    .collect();
                use VisualTableMenuItem as Item;
                match (column, action) {
                    (false, Item::InsertBefore) => {
                        assert_eq!(grid[2], ["", "", ""]);
                        assert_eq!(grid[3], ["a2", "b2", "c2"]);
                    }
                    (false, Item::InsertAfter) => {
                        assert_eq!(grid[2], ["a2", "b2", "c2"]);
                        assert_eq!(grid[3], ["", "", ""]);
                    }
                    (false, Item::MoveBefore) => assert_eq!(grid[1], ["a2", "b2", "c2"]),
                    (false, Item::MoveAfter) => assert_eq!(grid[3], ["a2", "b2", "c2"]),
                    (false, Item::Duplicate) => assert_eq!(grid[3], ["a2", "b2", "c2"]),
                    (false, Item::Clear) => assert_eq!(grid[2], ["", "", ""]),
                    (false, Item::Delete) => {
                        assert_eq!(grid.len(), 3);
                        assert_eq!(grid[2], ["a3", "b3", "c3"]);
                    }
                    (true, Item::InsertBefore) => assert_eq!(grid[2], ["a2", "", "b2", "c2"]),
                    (true, Item::InsertAfter) => assert_eq!(grid[2], ["a2", "b2", "", "c2"]),
                    (true, Item::MoveBefore) => assert_eq!(grid[2], ["b2", "a2", "c2"]),
                    (true, Item::MoveAfter) => assert_eq!(grid[2], ["a2", "c2", "b2"]),
                    (true, Item::Duplicate) => assert_eq!(grid[2], ["a2", "b2", "b2", "c2"]),
                    (true, Item::Clear) => assert_eq!(grid[2], ["a2", "", "c2"]),
                    (true, Item::Delete) => assert_eq!(grid[2], ["a2", "c2"]),
                    (true, Item::Align(alignment)) => assert_eq!(
                        alignments[1],
                        if *alignment == TableAlignment::Center {
                            TableAlignment::Default
                        } else {
                            *alignment
                        }
                    ),
                    _ => unreachable!(),
                }
                app.undo(&Undo, window, cx);
                assert_eq!(app.active_tab().document.text(), source);
                assert_eq!(app.active_tab().selected_range, 0..6);
            });
        }
    }
}

#[gpui::test]
fn combined_context_table_pointer_keyboard_and_guards(cx: &mut TestAppContext) {
    use markion::selection::CellSelectionAxis;
    let source = "before\n\n| A | B |\n| --- | --- |\n| one | two |\n| three | four |\n\nafter";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = 0..3;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    let index = app.read_with(cx, |app, _| {
        app.active_tab()
            .document
            .visual_blocks_shared()
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
            .unwrap()
    });
    let selector: &'static str =
        Box::leak(format!("visual-table-cell-{index}-2-1").into_boxed_str());
    let bounds = cx.debug_bounds(selector).unwrap();
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position: bounds.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        let menu = app.block_menu.as_ref().unwrap();
        assert_eq!(
            menu.table_context.unwrap().row.axis,
            VisualTableAxis::Row(2)
        );
        assert_eq!(
            menu.table_context.unwrap().column.axis,
            VisualTableAxis::Column(1)
        );
        assert_eq!(app.active_tab().selected_range, 0..3);
        let presentation = app.block_menu_presentation().unwrap();
        assert!(
            !presentation.item_enabled(BlockMenuItem::TableRow(VisualTableMenuItem::MoveAfter))
        );
        assert!(
            !presentation.item_enabled(BlockMenuItem::TableColumn(VisualTableMenuItem::MoveAfter))
        );
        activate_context_item(app, BlockMenuItem::CopySelection, None, cx);
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("bef")
        );
        app.select_table_cells(index, (1, 0), CellSelectionAxis::Rectangle, false, cx);
        app.select_table_cells(index, (2, 1), CellSelectionAxis::Rectangle, true, cx);
    });
    cx.run_until_parked();
    cx.dispatch_action(ShowVisualBlockContextMenu);
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        let table = app.block_menu.as_ref().unwrap().table_context.unwrap();
        assert_eq!(table.row.axis, VisualTableAxis::Row(2));
        assert_eq!(table.column.axis, VisualTableAxis::Column(1));
        app.close_visual_block_menu(cx);
        let blocks = app.active_tab().document.visual_blocks_shared();
        let target = BlockTarget::from_block(app.active_tab().document.version(), &blocks[index]);
        app.open_visual_pointer_context_menu(target, bounds.center(), Some((0, 0)), cx);
        for item in [
            BlockMenuItem::TableRow(VisualTableMenuItem::InsertBefore),
            BlockMenuItem::TableRow(VisualTableMenuItem::MoveBefore),
            BlockMenuItem::TableRow(VisualTableMenuItem::MoveAfter),
            BlockMenuItem::TableRow(VisualTableMenuItem::Delete),
            BlockMenuItem::TableColumn(VisualTableMenuItem::MoveBefore),
        ] {
            assert!(!app.block_menu_presentation().unwrap().item_enabled(item));
            open_context_submenu_for(app, item, cx);
            activate_context_item(app, item, None, cx);
            assert_eq!(app.active_tab().document.text(), source);
            assert!(app.active_tab().undo_stack.is_empty());
        }
        app.block_menu
            .as_mut()
            .unwrap()
            .table_context
            .as_mut()
            .unwrap()
            .column
            .column_count += 1;
        open_context_submenu_for(
            app,
            BlockMenuItem::TableColumn(VisualTableMenuItem::InsertAfter),
            cx,
        );
        activate_context_item(
            app,
            BlockMenuItem::TableColumn(VisualTableMenuItem::InsertAfter),
            None,
            cx,
        );
        assert_eq!(app.active_tab().document.text(), source);
        assert!(app.active_tab().undo_stack.is_empty());
    });
}

#[gpui::test]
fn editing_context_settings_follow_selection_owner(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for command in [
        SlashPaletteCommand::Link,
        SlashPaletteCommand::Formula,
        SlashPaletteCommand::Image,
    ] {
        for selection in [1..4, 2..2] {
            app.update_in(cx, |app, window, cx| {
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
                    "first\n\nsecond",
                ))];
                app.view_mode = ViewMode::VisualEdit;
                app.active_tab_mut().selected_range = selection.clone();
                let target = prose_target(app, "second");
                app.open_visual_edit_context_menu(target, point(px(40.), px(50.)), cx);
                activate_context_item(
                    app,
                    if command == SlashPaletteCommand::Link {
                        BlockMenuItem::SelectionFormat(SelectionFormatAction::Link)
                    } else {
                        BlockMenuItem::Insert(command)
                    },
                    Some(window),
                    cx,
                );
                let range = app
                    .link_editor
                    .as_ref()
                    .map(|editor| editor.source_range.clone())
                    .or_else(|| {
                        app.token_editor
                            .as_ref()
                            .map(|editor| editor.source_range.clone())
                    })
                    .unwrap();
                assert_eq!(range, selection);
                app.cancel_link_editor(cx);
                app.cancel_token_editor(cx);
                assert_eq!(app.active_tab().selected_range, selection);
                assert_eq!(app.active_tab().document.text(), "first\n\nsecond");
                assert!(app.active_tab().undo_stack.is_empty());
            });
        }
    }
}

#[gpui::test]
fn editing_context_table_cells_use_structured_content(cx: &mut TestAppContext) {
    use markion::selection::{CellSelectionAxis, SelectionState};
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for action in [
        BlockMenuItem::CopySelection,
        BlockMenuItem::CutSelection,
        BlockMenuItem::DeleteSelection,
        BlockMenuItem::Paste,
    ] {
        app.update_in(cx, |app, window, cx| {
            let source = "before\n\n| A | B |\n| --- | --- |\n| cell | keep |\n\nafter";
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
            app.view_mode = ViewMode::VisualEdit;
            let blocks = app.active_tab().document.visual_blocks_shared();
            let index = blocks
                .iter()
                .position(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
                .unwrap();
            app.select_table_cells(index, (1, 0), CellSelectionAxis::Rectangle, false, cx);
            let captured = app.active_tab().structured_selection.clone();
            let SelectionState::Cells(selection) = captured.as_ref().unwrap() else {
                panic!("cell selection")
            };
            let expected_copy = selection
                .clipboard(&app.active_tab().document)
                .unwrap()
                .plain;
            let target = prose_target(app, "after");
            app.open_visual_edit_context_menu(target, point(px(40.), px(50.)), cx);
            assert!(app.block_menu_presentation().unwrap().item_enabled(action));
            cx.write_to_clipboard(ClipboardItem::new_string("replaced".into()));
            activate_context_item(app, action, Some(window), cx);
            if matches!(
                action,
                BlockMenuItem::CopySelection | BlockMenuItem::CutSelection
            ) {
                assert_eq!(
                    cx.read_from_clipboard().unwrap().text().unwrap(),
                    expected_copy
                );
            }
            if action == BlockMenuItem::CopySelection {
                assert_eq!(app.active_tab().document.text(), source);
                assert_eq!(app.active_tab().structured_selection, captured);
                let target = prose_target(app, "after");
                app.open_visual_edit_context_menu(target, point(px(40.), px(50.)), cx);
                if let Some(SelectionState::Cells(selection)) =
                    &mut app.active_tab_mut().structured_selection
                {
                    selection.head = (1, 1);
                }
                cx.write_to_clipboard(ClipboardItem::new_string("unchanged".into()));
                activate_context_item(app, BlockMenuItem::CopySelection, Some(window), cx);
                assert!(app.block_menu.is_none());
                assert_eq!(
                    cx.read_from_clipboard().unwrap().text().as_deref(),
                    Some("unchanged")
                );
            } else {
                let replacement = if action == BlockMenuItem::Paste {
                    "replaced"
                } else {
                    ""
                };
                assert_eq!(
                    app.active_tab().document.text(),
                    source.replace("cell", replacement)
                );
                assert_eq!(app.active_tab().undo_stack.len(), 1);
                app.undo(&Undo, window, cx);
                assert_eq!(app.active_tab().document.text(), source);
                assert!(matches!(
                    app.active_tab().structured_selection,
                    Some(SelectionState::Cells(_))
                ));
            }
        });
    }
}

fn prose_target(app: &MarkionApp, needle: &str) -> BlockTarget {
    let doc = &app.active_tab().document;
    let blocks = doc.visual_blocks_shared();
    let block = blocks
        .iter()
        .find(|block| {
            matches!(
                block.kind,
                VisualBlockKind::Paragraph | VisualBlockKind::Heading { .. }
            ) && doc.text()[block.source_range.clone()].contains(needle)
        })
        .unwrap();
    BlockTarget::from_block(doc.version(), block)
}

#[gpui::test]
fn compact_context_direct_controls_match_palette_and_preserve_target(cx: &mut TestAppContext) {
    let source = "first\r\n\r\n## target 中文\r\n\r\nafter";
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
    app.update(cx, |app, cx| {
        let target = prose_target(app, "target");
        let blocks = app.active_tab().document.visual_blocks_shared();
        app.open_visual_block_menu(target, point(px(100.), px(100.)), cx);
        assert_eq!(app.active_tab().selected_range, 1..1);
        assert_eq!(app.active_tab().document.text(), source);
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
        let items = app.block_menu.as_ref().unwrap().root_items();
        assert_eq!(items.len(), 24);
        assert!(!items.contains(&BlockMenuItem::Transform(BlockTransform::Heading(6))));
        for action in [
            SlashPaletteCommand::Link,
            SlashPaletteCommand::Formula,
            SlashPaletteCommand::Image,
        ] {
            assert!(items.contains(&BlockMenuItem::Insert(action)));
        }
        for action in BLOCK_MENU_OPERATIONS {
            assert!(items.contains(&action));
        }
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let target_index = blocks
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Heading { .. }))
            .unwrap();
        let bounds = app
            .active_tab()
            .visual_list
            .bounds_for_item(target_index)
            .unwrap();
        let caret = Bounds::new(point(px(9.), px(15.)), size(px(2.), px(20.)));
        app.active_tab_mut().visual_caret_bounds = Some(caret);
        assert_eq!(
            app.document_overlay_anchor(Some(target_index), overlay::OverlayPlacement::BelowSource),
            point(bounds.left(), bounds.bottom()),
            "settings must follow the clicked block while preserving an unrelated caret"
        );
        assert_eq!(
            app.document_overlay_anchor(Some(0), overlay::OverlayPlacement::BelowSource),
            point(caret.left(), caret.bottom())
        );
        assert_eq!(app.active_tab().selected_range, 1..1);
    });
    let panel = cx.debug_bounds("visual-block-menu-panel").unwrap();
    assert_eq!(panel.size.width, px(248.));
    let h1 = cx.debug_bounds("visual-block-transform-1").unwrap();
    let h2 = cx.debug_bounds("visual-block-transform-2").unwrap();
    let h5 = cx.debug_bounds("visual-block-transform-5").unwrap();
    assert_eq!(h1.top(), h5.top());
    assert_eq!(h1.size, size(px(36.), px(30.)));
    assert_eq!(h2.left() - h1.right(), px(2.));
    assert!(cx.debug_bounds("visual-block-transform-6").is_none());
    let task = cx.debug_bounds("visual-block-transform-9").unwrap();
    cx.simulate_click(task.center(), Modifiers::none());
    cx.run_until_parked();
    app.update_in(cx, |app, window, cx| {
        assert_eq!(
            app.active_tab().document.text(),
            "first\r\n\r\n- [ ] target 中文\r\n\r\nafter"
        );
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        app.undo(&Undo, window, cx);
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().selected_range, 1..1);
    });
}

#[gpui::test]
fn compact_context_settings_target_clicked_block_and_cancel_without_mutation(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for command in [
        SlashPaletteCommand::Link,
        SlashPaletteCommand::Formula,
        SlashPaletteCommand::Image,
    ] {
        for selection in [0..3, 11..13, 11..11] {
            app.update_in(cx, |app, window, cx| {
                let source = "first\r\n\r\ntarget 中文\r\n\r\nafter";
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
                app.view_mode = ViewMode::VisualEdit;
                app.slash_commands_enabled = false;
                app.active_tab_mut().selected_range = selection.clone();
                let target = prose_target(app, "target");
                let end = source.find(" 中文").unwrap() + " 中文".len();
                let expected = if selection.start < 9 {
                    end..end
                } else {
                    selection.clone()
                };
                app.open_visual_block_menu(target, point(px(40.), px(50.)), cx);
                let version = app.active_tab().document.version();
                let blocks = app.active_tab().document.visual_blocks_shared();
                activate_context_item(app, BlockMenuItem::Insert(command), Some(window), cx);
                let range = app
                    .link_editor
                    .as_ref()
                    .map(|editor| editor.source_range.clone())
                    .or_else(|| {
                        app.token_editor
                            .as_ref()
                            .map(|editor| editor.source_range.clone())
                    })
                    .unwrap();
                assert_eq!(range, expected);
                assert!(app.block_menu.is_none());
                assert_eq!(app.active_tab().document.text(), source);
                assert_eq!(app.active_tab().selected_range, selection);
                assert_eq!(app.active_tab().document.version(), version);
                assert!(app.active_tab().undo_stack.is_empty());
                assert!(Arc::ptr_eq(
                    &blocks,
                    &app.active_tab().document.visual_blocks_shared()
                ));
                app.cancel_link_editor(cx);
                app.cancel_token_editor(cx);
                assert_eq!(app.active_tab().selected_range, selection);
            });
        }
    }
}

#[gpui::test]
fn compact_context_link_formula_confirm_and_paste_preserve_unrelated_source(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for ending in ["\n", "\r\n"] {
        for command in [SlashPaletteCommand::Link, SlashPaletteCommand::Formula] {
            app.update_in(cx, |app, window, cx| {
                let source = format!("first{ending}{ending}target{ending}{ending}after");
                let end = source.find("target").unwrap() + 6;
                app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
                app.view_mode = ViewMode::VisualEdit;
                app.active_tab_mut().selected_range = 0..3;
                let target = prose_target(app, "target");
                app.open_visual_block_menu(target, point(px(40.), px(50.)), cx);
                activate_context_item(app, BlockMenuItem::Insert(command), Some(window), cx);
                let inserted = if command == SlashPaletteCommand::Link {
                    let editor = app.link_editor.as_mut().unwrap();
                    editor.label = "文档".into();
                    editor.url = "https://example.test/guide".into();
                    app.confirm_link_editor(window, cx);
                    "[文档](https://example.test/guide)"
                } else {
                    app.token_editor.as_mut().unwrap().draft = "x^2".into();
                    app.confirm_token_editor(cx);
                    "$x^2$"
                };
                assert_eq!(
                    app.active_tab().document.text(),
                    format!("{}{inserted}{}", &source[..end], &source[end..])
                );
                assert_eq!(app.active_tab().undo_stack.len(), 1);
                app.undo(&Undo, window, cx);
                assert_eq!(app.active_tab().document.text(), source);

                let target = prose_target(app, "target");
                app.open_visual_block_menu(target, point(px(40.), px(50.)), cx);
                app.file_tree_query_focused = true;
                app.file_tree_query = "preserved query".into();
                cx.write_to_clipboard(ClipboardItem::new_string(" pasted 中文".into()));
                activate_context_item(app, BlockMenuItem::Paste, Some(window), cx);
                assert_eq!(
                    app.active_tab().document.text(),
                    format!("{} pasted 中文{}", &source[..end], &source[end..])
                );
                assert_eq!(app.active_tab().undo_stack.len(), 1);
                assert_eq!(app.file_tree_query, "preserved query");
                assert!(app.file_tree_query_focused);
                app.undo(&Undo, window, cx);
                assert_eq!(app.active_tab().document.text(), source);
            });
        }
    }
}

#[gpui::test]
fn compact_context_keyboard_reaches_every_enabled_row_in_short_windows(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("one\n\ntwo"))];
        app.view_mode = ViewMode::VisualEdit;
        app.sidebar_visible = false;
        app
    });
    cx.simulate_resize(size(px(340.), px(260.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for (theme, editing) in [
        (AppTheme::Paper, false),
        (AppTheme::Ink, false),
        (AppTheme::Paper, true),
        (AppTheme::Ink, true),
    ] {
        app.update(cx, |app, cx| {
            app.theme = theme;
            let target = prose_target(app, "one");
            if editing {
                app.active_tab_mut().selected_range = 0..2;
                app.open_visual_edit_context_menu(target, point(px(335.), px(255.)), cx);
            } else {
                app.open_visual_block_menu(target, point(px(335.), px(255.)), cx);
            }
        });
        cx.run_until_parked();
        let count = app.read_with(cx, |app, _| {
            app.block_menu.as_ref().unwrap().root_items().len()
        });
        for _ in 0..count {
            cx.dispatch_action(Down);
            cx.run_until_parked();
            let item = app.read_with(cx, |app, _| {
                let menu = app.block_menu.as_ref().unwrap();
                menu.root_items()[menu.root_selected]
            });
            let button = cx
                .debug_bounds(super::preview::block_menu_item_element_id(item))
                .unwrap();
            let panel = cx.debug_bounds("visual-block-menu-panel").unwrap();
            assert!(
                panel.contains(&button.center()),
                "{item:?}: {button:?} outside {panel:?}"
            );
        }
        app.update(cx, |app, _| {
            assert_eq!(app.active_tab().document.text(), "one\n\ntwo");
            assert!(app.active_tab().undo_stack.is_empty());
        });
        cx.dispatch_action(ClearFileTreeSearch);
    }
}

#[gpui::test]
fn compact_context_stale_target_or_changed_selection_rejects_insertion(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    for command in [
        SlashPaletteCommand::Link,
        SlashPaletteCommand::Formula,
        SlashPaletteCommand::Image,
    ] {
        app.update_in(cx, |app, window, cx| {
            let source = "selected prose\n\nother";
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
            app.view_mode = ViewMode::VisualEdit;
            app.active_tab_mut().selected_range = 0..8;
            let target = prose_target(app, "selected");
            app.open_visual_block_menu(target, point(px(20.), px(20.)), cx);
            app.active_tab_mut().selected_range = 3..3;
            activate_context_item(app, BlockMenuItem::Insert(command), Some(window), cx);
            assert!(app.block_menu.is_none());
            assert!(app.link_editor.is_none() && app.token_editor.is_none());
            assert_eq!(app.active_tab().document.text(), source);
            assert!(app.active_tab().undo_stack.is_empty());

            let target = prose_target(app, "selected");
            app.open_visual_block_menu(target, point(px(20.), px(20.)), cx);
            activate_context_item(app, BlockMenuItem::Insert(command), Some(window), cx);
            let mutation = app
                .active_tab()
                .document
                .prepare_whole_mutation(MutationOrigin::ExternalReload, "external change");
            assert!(
                app.apply_document_mutation("context_settings_external_change", mutation)
                    .is_some()
            );
            if command == SlashPaletteCommand::Formula {
                app.token_editor.as_mut().unwrap().draft = "x^2".into();
                app.confirm_token_editor(cx);
            } else {
                app.link_editor.as_mut().unwrap().url = "https://example.test/image.png".into();
                app.confirm_link_editor(window, cx);
            }
            assert_eq!(app.active_tab().document.text(), "external change");
            assert!(app.active_tab().undo_stack.is_empty());
            assert!(app.link_editor.is_none() && app.token_editor.is_none());
        });
    }
}

#[gpui::test]
fn compact_context_image_settings_keep_resource_policy_and_target(cx: &mut TestAppContext) {
    let source = "first\r\n\r\ntarget\r\n\r\nafter";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app.image_preferences.remote_policy = markion::RemoteImagePolicy::Keep;
        app
    });
    app.update_in(cx, |app, window, cx| {
        let target = prose_target(app, "target");
        app.open_visual_block_menu(target, point(px(20.), px(20.)), cx);
        activate_context_item(
            app,
            BlockMenuItem::Insert(SlashPaletteCommand::Image),
            Some(window),
            cx,
        );
        let editor = app.link_editor.as_mut().unwrap();
        editor.label = "图".into();
        editor.url = "https://example.test/image.png".into();
        editor.title = "caption".into();
        app.confirm_link_editor(window, cx);
    });
    super::tests::wait_for_image_operations(&app, cx);
    app.update_in(cx, |app, window, cx| {
        assert_eq!(
            app.active_tab().document.text(),
            "first\r\n\r\ntarget![图](https://example.test/image.png \"caption\")\r\n\r\nafter"
        );
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        app.undo(&Undo, window, cx);
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().selected_range, 1..1);
    });
}
