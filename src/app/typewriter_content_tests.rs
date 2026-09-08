use super::*;
use gpui::{ListOffset, TestAppContext};

fn assert_content_frames(
    app: &Entity<MarkionApp>,
    window: &mut Window,
    cx: &mut App,
    action: &str,
) {
    let before = app.read(cx).active_tab();
    let version = before.document.version();
    let dirty = before.document.is_dirty();
    let undo_len = before.undo_stack.len();
    let text = before.document.text().to_owned();
    let visual = before.document.visual_blocks_shared();
    let preview = before.document.preview_blocks_shared();
    let text_handle = before.shared_document_text();
    let highlighted = app
        .read(cx)
        .highlighted_code(None, "typewriter cache probe");
    let highlight_count = app.read(cx).highlight_cache.borrow().len();
    for frame in 0..TYPEWRITER_VISUAL_REFINEMENT_FRAMES + 3 {
        app.update(cx, |app, _| {
            app.active_tab_mut().visual_text_paints = Some(Vec::new());
        });
        let caret_paints = app.read(cx).active_tab().visual_caret_paint_count;
        window.refresh();
        let _ = window.draw(cx);
        let app = app.read(cx);
        let tab = app.active_tab();
        let viewport = tab.visual_list.viewport_bounds();
        let caret = tab.visual_caret_bounds.expect("current caret");
        assert!(tab.visual_caret_paint_count > caret_paints);
        let delta = typewriter_center_delta(viewport, caret).unwrap();
        assert!(
            f32::from(delta).abs() <= 1.,
            "{action} frame {frame}: {delta:?}"
        );

        // The fixture starts with one plain text line. Its expected position
        // comes from content-space geometry, independently of bounds_for_item
        // (which itself skips predecessors of the faulty logical anchor).
        let line_height = px(app.typography_metrics().paragraph_line_height);
        let first_top = viewport.top()
            + tab.visual_end_padding_height.unwrap()
            + tab.visual_list.scroll_px_offset_for_scrollbar().y;
        if first_top >= viewport.top() && first_top + line_height <= viewport.bottom() {
            let center = first_top + line_height / 2.;
            assert!(
                tab.visual_text_paints
                    .as_ref()
                    .unwrap()
                    .iter()
                    .any(|(ix, bounds)| {
                        *ix == 0
                            && bounds.top() <= center
                            && bounds.bottom() >= center
                            && bounds.size.height > px(0.)
                    }),
                "{action} frame {frame}: preceding text must paint at {center:?}; anchor={:?}, paints={:?}",
                tab.visual_list.logical_scroll_top(),
                tab.visual_text_paints,
            );
        }
        assert_eq!(tab.document.text(), text);
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.is_dirty(), dirty);
        assert_eq!(tab.undo_stack.len(), undo_len);
        assert!(Arc::ptr_eq(&visual, &tab.document.visual_blocks_shared()));
        assert!(Arc::ptr_eq(&preview, &tab.document.preview_blocks_shared()));
        assert_eq!(text_handle.as_ptr(), tab.shared_document_text().as_ptr());
        assert_eq!(app.highlight_cache.borrow().len(), highlight_count);
        assert!(Rc::ptr_eq(
            &highlighted,
            &app.highlighted_code(None, "typewriter cache probe")
        ));
    }
}

#[gpui::test]
fn visual_typewriter_content_existing_paragraphs(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(""))];
        app.view_mode = ViewMode::VisualEdit;
        app.typewriter_mode = true;
        app.sidebar_visible = false;
        app
    });
    cx.simulate_resize(size(px(640.), px(400.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    // Replay the history that exposed the defect, then continue ordinary
    // text + single-Enter cycles in the later paragraph.
    for input in [
        "测试", "\n", "\n", "\n", "测试", "\n", "测试", "\n", "测试", "\n",
    ] {
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                if input == "\n" {
                    app.insert_newline(&InsertNewline, window, cx);
                } else {
                    EntityInputHandler::replace_text_in_range(app, None, input, window, cx);
                }
            });
            assert_content_frames(&app, window, cx, input);
        });
    }
}

struct AutoscrollList {
    state: ListState,
    heights: Vec<f32>,
    request: Option<(usize, f32, f32)>,
    painted: Rc<RefCell<Vec<(usize, Bounds<Pixels>)>>>,
    padding: f32,
}

impl Render for AutoscrollList {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let heights = self.heights.clone();
        let request = self.request;
        let painted = self.painted.clone();
        gpui::list(self.state.clone(), move |ix, _, _| {
            let painted = painted.clone();
            gpui::canvas(
                move |bounds, window, _| {
                    if let Some((target, relative_top, height)) = request
                        && target == ix
                    {
                        window.request_autoscroll(Bounds::new(
                            point(bounds.left(), bounds.top() + px(relative_top)),
                            size(px(1.), px(height)),
                        ));
                    }
                },
                move |bounds, _, window, _| {
                    painted
                        .borrow_mut()
                        .push((ix, bounds.intersect(&window.content_mask().bounds)));
                },
            )
            .w_full()
            .h(px(heights[ix]))
            .into_any_element()
        })
        .size_full()
        .pt(px(self.padding))
    }
}

#[gpui::test]
fn list_autoscroll_content_includes_predecessors(cx: &mut TestAppContext) {
    for padding in [0., 10.] {
        for cached in [false, true] {
            assert_list_case(cx, padding, cached, 2, (3, -100., 200. - padding), 40.);
            assert_list_case(cx, padding, cached, 2, (3, -280., 200. - padding), 0.);
            assert_list_case(cx, padding, cached, 0, (3, 0., 120.), 60. + padding);
        }
    }
}

fn assert_list_case(
    cx: &mut TestAppContext,
    padding: f32,
    cached: bool,
    start: usize,
    request: (usize, f32, f32),
    expected_offset: f32,
) {
    let (app, cx) = cx.add_window_view(|_, _| {
        let state = ListState::new(5, ListAlignment::Top, px(0.));
        AutoscrollList {
            state,
            heights: vec![80., 0., 60., 120., 200.],
            request: None,
            painted: Rc::new(RefCell::new(Vec::new())),
            padding,
        }
    });
    cx.simulate_resize(size(px(300.), px(200.)));
    cx.update(|window, cx| {
        window.refresh();
        let _ = window.draw(cx);
        app.update(cx, |app, _| {
            if !cached {
                app.state = ListState::new(app.heights.len(), ListAlignment::Top, px(0.));
            }
            app.state.scroll_to(ListOffset { item_ix: start, offset_in_item: px(0.) });
            app.request = Some(request);
        });
        app.read(cx).painted.borrow_mut().clear();
        window.refresh();
        let _ = window.draw(cx);
        let app = app.read(cx);
        let viewport = app.state.viewport_bounds();
        let mut top = viewport.top() + px(padding - expected_offset);
        for (ix, height) in app.heights.iter().enumerate() {
            let expected = Bounds::new(point(viewport.left(), top), size(viewport.size.width, px(*height)))
                .intersect(&viewport);
            if expected.size.height > px(0.) {
                let painted = app.painted.borrow();
                let actual = painted.iter().find(|(row, _)| *row == ix)
                    .unwrap_or_else(|| panic!("row {ix} must paint: padding={padding}, cached={cached}, request={request:?}, anchor={:?}, paints={painted:?}", app.state.logical_scroll_top()));
                assert_eq!(actual.1, expected);
            }
            top += px(*height);
        }
        assert_eq!(app.state.scroll_px_offset_for_scrollbar().y, px(-expected_offset));
    });
}

#[gpui::test]
fn list_autoscroll_content_preserves_short_bottom_alignment(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| AutoscrollList {
        state: ListState::new(2, ListAlignment::Bottom, px(0.)),
        heights: vec![20., 30.],
        request: None,
        painted: Rc::new(RefCell::new(Vec::new())),
        padding: 0.,
    });
    cx.simulate_resize(size(px(300.), px(200.)));
    cx.update(|window, cx| {
        app.read(cx).painted.borrow_mut().clear();
        window.refresh();
        let _ = window.draw(cx);
        let app = app.read(cx);
        assert_eq!(app.painted.borrow().len(), 2);
        assert_eq!(
            app.painted.borrow()[0].1.top(),
            app.state.viewport_bounds().bottom() - px(50.)
        );
        assert_eq!(
            app.painted.borrow()[1].1.bottom(),
            app.state.viewport_bounds().bottom()
        );
    });
}

#[gpui::test]
fn visual_typewriter_content_seeded_two_paragraphs(cx: &mut TestAppContext) {
    for (width, height, font_size, spacing) in [(640., 400., 14, 12), (480., 300., 15, 13)] {
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            let source = "测试\n\n测试\n";
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
            app.view_mode = ViewMode::VisualEdit;
            app.typewriter_mode = true;
            app.sidebar_visible = false;
            app.rendered_font_size = font_size;
            app.paragraph_spacing = spacing;
            app.active_tab_mut().selected_range = source.len()..source.len();
            app
        });
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                EntityInputHandler::replace_text_in_range(app, None, "测试", window, cx);
            });
            assert_content_frames(&app, window, cx, "first input in seeded document");
        });
        for _ in 0..3 {
            cx.update(|window, cx| {
                app.update(cx, |app, cx| app.insert_newline(&InsertNewline, window, cx));
                assert_content_frames(&app, window, cx, "single Enter");
            });
            for composition in ["c", "ce", "ceshi"] {
                cx.update(|window, cx| {
                    app.update(cx, |app, cx| {
                        EntityInputHandler::replace_and_mark_text_in_range(
                            app,
                            None,
                            composition,
                            None,
                            window,
                            cx,
                        );
                    });
                    assert_content_frames(&app, window, cx, composition);
                });
            }
            cx.update(|window, cx| {
                app.update(cx, |app, cx| {
                    EntityInputHandler::replace_text_in_range(app, None, "测试", window, cx);
                });
                assert_content_frames(&app, window, cx, "IME commit");
            });
        }
        cx.simulate_resize(size(px(width - 60.), px(height - 40.)));
        cx.update(|window, cx| {
            assert_content_frames(&app, window, cx, "resize");
            app.update(cx, |app, cx| {
                app.set_rendered_font_size(16, cx);
                app.set_paragraph_spacing(16, cx);
            });
            assert_content_frames(&app, window, cx, "typography change");
            app.update(cx, |app, cx| {
                EntityInputHandler::replace_text_in_range(
                    app,
                    None,
                    &"wrapped words ".repeat(80),
                    window,
                    cx,
                );
            });
            assert_content_frames(&app, window, cx, "wrapped paragraph");
        });
    }
}
