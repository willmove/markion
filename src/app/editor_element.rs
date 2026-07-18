use super::*;

pub(super) fn visual_ime_bounds(
    caret: Option<Bounds<Pixels>>,
    surface: Option<Bounds<Pixels>>,
) -> Option<Bounds<Pixels>> {
    caret.or_else(|| {
        surface.map(|surface| {
            Bounds::new(
                point(
                    surface.left() + px(PANE_INNER_PADDING),
                    surface.top() + px(PANE_INNER_PADDING),
                ),
                size(px(2.), px(PREVIEW_LINE_HEIGHT)),
            )
        })
    })
}

impl EntityInputHandler for MarkionApp {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let tab = self.active_tab();
        let range = tab.range_from_utf16(&range_utf16);
        actual_range.replace(tab.range_to_utf16(&range));
        Some(tab.document.text()[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let tab = self.active_tab();
        Some(UTF16Selection {
            range: tab.range_to_utf16(&tab.selected_range),
            reversed: tab.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        let tab = self.active_tab();
        tab.marked_range
            .as_ref()
            .map(|range| tab.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        tab.marked_range = None;
        tab.finish_undo_capture();
        self.input_marked_len = 0;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.has_text_input_focus() {
            self.push_text_input(new_text, cx);
            return;
        }

        let visual_edit = matches!(self.view_mode, ViewMode::VisualEdit);
        let tab = self.active_tab_mut();
        let active_marked_range = tab.marked_range.clone();
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| tab.range_from_utf16(range_utf16))
            .or(active_marked_range.clone())
            .unwrap_or(tab.selected_range.clone());

        let direct_edit = visual_edit
            .then(|| {
                tab.document
                    .direct_visual_block_edit(range.clone(), new_text)
            })
            .flatten()
            .filter(|edit| tab.document.validate_visual_block_edit(edit));
        let edit_range = direct_edit
            .as_ref()
            .map_or_else(|| range.clone(), |edit| edit.range.clone());
        let replacement = direct_edit
            .as_ref()
            .map_or_else(|| new_text.to_string(), |edit| edit.replacement.clone());

        let changed = tab.document.text()[edit_range.clone()] != replacement;
        let committing_ime = active_marked_range.is_some()
            && tab
                .undo_capture
                .is_some_and(|capture| capture.kind == UndoCaptureKind::Ime);
        if changed {
            let intent = if committing_ime {
                UndoCaptureKind::Ime
            } else {
                tab.pending_text_edit_intent.take().unwrap_or_else(|| {
                    if range.is_empty() && !new_text.is_empty() {
                        UndoCaptureKind::Insert
                    } else {
                        UndoCaptureKind::Atomic
                    }
                })
            };
            let history_range = direct_edit
                .as_ref()
                .map_or_else(|| edit_range.clone(), |_| range.clone());
            let history_replacement = if direct_edit.is_some() {
                new_text
            } else {
                &replacement
            };
            tab.prepare_undo_capture(intent, &history_range, history_replacement, Instant::now());
            if let (Some(edit), Some(capture)) = (direct_edit.as_ref(), tab.undo_capture.as_mut()) {
                capture.next_cursor = edit.selection_after.end;
            }
            tab.document.replace_range(edit_range.clone(), &replacement);
        }
        tab.selected_range = direct_edit.as_ref().map_or_else(
            || range.start + replacement.len()..range.start + replacement.len(),
            |edit| edit.selection_after.clone(),
        );
        tab.marked_range.take();
        if committing_ime {
            tab.finish_undo_capture();
        }
        self.status = t(
            self.language,
            if changed {
                Msg::StatusEditing
            } else {
                Msg::StatusNoEdit
            },
        )
        .into();
        if changed {
            self.after_document_changed(cx);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.has_text_input_focus() {
            self.insert_redirected_text(new_text, true, cx);
            return;
        }

        let visual_edit = matches!(self.view_mode, ViewMode::VisualEdit);
        let tab = self.active_tab_mut();
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| tab.range_from_utf16(range_utf16))
            .or(tab.marked_range.clone())
            .unwrap_or(tab.selected_range.clone());

        let direct_edit = visual_edit
            .then(|| {
                tab.document
                    .direct_visual_block_edit(range.clone(), new_text)
            })
            .flatten()
            .filter(|edit| tab.document.validate_visual_block_edit(edit));
        let edit_range = direct_edit
            .as_ref()
            .map_or_else(|| range.clone(), |edit| edit.range.clone());
        let replacement = direct_edit
            .as_ref()
            .map_or_else(|| new_text.to_string(), |edit| edit.replacement.clone());

        let changed = tab.document.text()[edit_range.clone()] != replacement;
        if changed {
            let history_range = direct_edit
                .as_ref()
                .map_or_else(|| edit_range.clone(), |_| range.clone());
            let history_replacement = if direct_edit.is_some() {
                new_text
            } else {
                &replacement
            };
            tab.prepare_undo_capture(
                UndoCaptureKind::Ime,
                &history_range,
                history_replacement,
                Instant::now(),
            );
            if let (Some(edit), Some(capture)) = (direct_edit.as_ref(), tab.undo_capture.as_mut()) {
                capture.next_cursor = edit.selection_after.end;
            }
            tab.document.replace_range(edit_range.clone(), &replacement);
        }
        tab.marked_range = direct_edit.as_ref().map_or_else(
            || {
                (!replacement.is_empty())
                    .then_some(edit_range.start..edit_range.start + replacement.len())
            },
            |edit| {
                (!edit.inserted_range_after.is_empty()).then_some(edit.inserted_range_after.clone())
            },
        );
        tab.selected_range = if let Some(edit) = direct_edit.as_ref() {
            edit.selection_after.clone()
        } else {
            new_selected_range_utf16
                .as_ref()
                .map(|range_utf16| EditorTab::relative_range_from_utf16(new_text, range_utf16))
                .map(|new_range| new_range.start + range.start..new_range.end + range.start)
                .unwrap_or_else(|| range.start + replacement.len()..range.start + replacement.len())
        };
        self.status = t(self.language, Msg::StatusComposing).into();
        if changed {
            self.after_document_changed(cx);
        }
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let tab = self.active_tab();
        if matches!(self.view_mode, ViewMode::VisualEdit) {
            let source_range = tab.range_from_utf16(&range_utf16);
            if let Some((marked_range, marked_bounds)) = &tab.visual_marked_range_bounds
                && source_range.start <= marked_range.end
                && source_range.end >= marked_range.start
            {
                return Some(*marked_bounds);
            }
            return visual_ime_bounds(tab.visual_caret_bounds, tab.visual_input_bounds);
        }
        if tab.last_lines.is_empty() {
            return None;
        }
        let range = tab.range_from_utf16(&range_utf16);
        let line_height = tab.line_height;
        let start = tab.layout_point_for_offset(range.start, bounds, line_height)?;
        let end = tab.layout_point_for_offset(range.end, bounds, line_height)?;
        Some(Bounds::from_corners(start, end))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let tab = self.active_tab();
        if tab.last_lines.is_empty() {
            return None;
        }
        let utf8_index = tab.index_for_mouse_position(point);
        Some(tab.offset_to_utf16(utf8_index))
    }
}

pub(super) struct EditorElement {
    pub(super) app: gpui::Entity<MarkionApp>,
}

pub(super) struct PrepaintState {
    lines: Vec<WrappedLine>,
    line_offsets: Vec<usize>,
    line_heights: Vec<Pixels>,
    cursors: Vec<PaintQuad>,
    selections: Vec<PaintQuad>,
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.).into();
        // Measure the editor's height from the wrapped text layout so soft
        // wrapped lines are fully scrollable. A plain `line_count *
        // line_height` reservation undercounts wrapped lines and clips the
        // bottom of the document. Wrap boundaries only depend on the font, so
        // a single plain run measures identically to the styled paint runs and
        // hits the same per-line layout cache.
        let text_style = window.text_style();
        let line_height = window.line_height();
        let app_entity = self.app.clone();
        let layout_id =
            window.request_measured_layout(style, move |known, available, window, cx| {
                let width = known.width.unwrap_or(match available.width {
                    gpui::AvailableSpace::Definite(width) => width,
                    _ => px(0.),
                });
                let app = app_entity.read(cx);
                let line_count = app.active_tab().document.line_count();
                let fallback = size(width, line_height * line_count as f32);
                if app.active_tab().document.text().is_empty() || width <= px(0.) {
                    return size(width, line_height);
                }
                let font_size = text_style.font_size.to_pixels(window.rem_size());
                // Skip the whole-document measure when nothing that affects the
                // wrapped height has changed since the last pass.
                let cache_key = MeasuredHeightKey {
                    version: app.active_tab().document.version(),
                    wrap_width: width,
                    font_size,
                    line_height,
                };
                if let Some((key, height)) = *app.active_tab().measured_height_cache.borrow()
                    && key == cache_key
                {
                    return size(width, height);
                }
                let text = app.shared_document_text();
                let run = TextRun {
                    len: text.len(),
                    font: text_style.font(),
                    color: text_style.color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let height = window
                    .text_system()
                    .shape_text(text, font_size, &[run], Some(width), None)
                    .map(|lines| {
                        lines
                            .iter()
                            .map(|line| line.size(line_height).height)
                            .fold(px(0.), |total, height| total + height)
                    });
                match height {
                    Ok(height) => {
                        let height = height.max(line_height);
                        *app.active_tab().measured_height_cache.borrow_mut() =
                            Some((cache_key, height));
                        size(width, height)
                    }
                    Err(_) => fallback,
                }
            });
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let app = self.app.read(cx);
        let tab = app.active_tab();
        let is_empty = tab.document.text().is_empty();
        let marked_range = tab.marked_range.clone();
        let cursor_offset = tab.cursor_offset();
        let selected_range = tab.selected_range.clone();
        let text: SharedString = if is_empty {
            SharedString::from("Type Markdown here...")
        } else {
            // Cached per document version; avoids copying the whole document
            // text on every frame.
            tab.shared_document_text()
        };
        let text_len = tab.document.text().len();
        let display_text = text;
        let style = window.text_style();
        let text_color = if is_empty {
            gpui::hsla(0., 0., 0.47, 0.67)
        } else {
            style.color
        };
        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len().saturating_sub(marked_range.end),
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else if app.focus_mode && !is_empty {
            let focus_range = tab.document.paragraph_range_at(cursor_offset);
            let muted_color = if app.theme == AppTheme::Ink {
                gpui::hsla(218., 0.18, 0.72, 0.42)
            } else {
                gpui::hsla(215., 0.16, 0.42, 0.38)
            };
            vec![
                TextRun {
                    len: focus_range.start,
                    color: muted_color,
                    ..run.clone()
                },
                TextRun {
                    len: focus_range.end.saturating_sub(focus_range.start),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len().saturating_sub(focus_range.end),
                    color: muted_color,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        // shape_text splits on `\n` and wraps each logical line to the editor
        // width, giving us one WrappedLine per source line.
        let wrap_width = bounds.size.width;
        let lines = window
            .text_system()
            .shape_text(
                display_text.clone(),
                font_size,
                &runs,
                Some(wrap_width),
                None,
            )
            .map(|lines| lines.into_iter().collect::<Vec<_>>())
            .unwrap_or_default();

        // Byte offset at the start of each logical line in the document,
        // cached per document version — rebuilding it is an O(document) scan.
        let line_offsets: Vec<usize> = if is_empty {
            Vec::new()
        } else {
            tab.shared_line_offsets().as_ref().clone()
        };

        let line_heights: Vec<Pixels> = lines
            .iter()
            .map(|line| line.size(line_height).height)
            .collect();

        let mut cursors = Vec::new();
        let mut selections = Vec::new();

        // Convert a document byte offset into a screen-space point by finding
        // the logical line that contains it and asking its layout for the
        // wrapped position within that line.
        let offset_to_point = |offset: usize| -> Option<(usize, Point<Pixels>)> {
            if is_empty || lines.is_empty() {
                return None;
            }
            let clamped = offset.min(text_len);
            // Find the logical line whose [start, end) range contains `offset`.
            let mut line_index = 0;
            for (i, &start) in line_offsets.iter().enumerate() {
                let end = line_offsets.get(i + 1).copied().unwrap_or(text_len + 1);
                if clamped >= start && clamped < end {
                    line_index = i;
                    break;
                }
            }
            let line = lines.get(line_index)?;
            let local_offset = clamped - line_offsets.get(line_index)?;
            let local = line.position_for_index(local_offset, line_height)?;
            let mut cumulative_y = px(0.);
            for i in 0..line_index {
                cumulative_y += line_heights.get(i).copied().unwrap_or(line_height);
            }
            Some((
                line_index,
                point(
                    bounds.left() + local.x,
                    bounds.top() + cumulative_y + local.y,
                ),
            ))
        };

        if selected_range.is_empty() {
            if let Some((_line_index, pos)) = offset_to_point(cursor_offset) {
                cursors.push(fill(
                    Bounds::new(pos, size(px(2.), line_height)),
                    rgb(0x2563eb),
                ));
            } else if is_empty {
                cursors.push(fill(
                    Bounds::new(
                        point(bounds.left(), bounds.top()),
                        size(px(2.), line_height),
                    ),
                    rgb(0x2563eb),
                ));
            }
        } else {
            // Build a selection quad for each logical line that the selection
            // range intersects.
            let start = selected_range.start;
            let end = selected_range.end;
            for (line_index, line) in lines.iter().enumerate() {
                let line_start = line_offsets.get(line_index).copied().unwrap_or(end);
                let line_end = line_offsets
                    .get(line_index + 1)
                    .map(|&next| next.saturating_sub(1))
                    .unwrap_or(text_len);
                if line_start >= end || line_end < start {
                    continue;
                }
                let line_len = line_end - line_start;
                let sel_start = start.max(line_start) - line_start;
                // The selection may cover this line's trailing newline; clamp
                // to the line text (position_for_index(len + 1) has no glyph)
                // and widen the final quad instead.
                let includes_newline = end > line_end && line_end < text_len;
                let sel_end = (end.min(line_end + 1) - line_start).min(line_len);

                let start_pos = line
                    .position_for_index(sel_start, line_height)
                    .unwrap_or(Point::default());
                let mut end_pos = line
                    .position_for_index(sel_end, line_height)
                    .unwrap_or(start_pos);
                if includes_newline {
                    end_pos.x += px(5.);
                }

                let mut cumulative_y = px(0.);
                for i in 0..line_index {
                    cumulative_y += line_heights.get(i).copied().unwrap_or(line_height);
                }
                let line_top = bounds.top() + cumulative_y;
                let selection_color = rgba(0x2563eb30);
                if start_pos.y == end_pos.y {
                    // Selection stays on one wrap row: a single quad from the
                    // start to the end position.
                    let top = line_top + start_pos.y;
                    selections.push(fill(
                        Bounds::from_corners(
                            point(bounds.left() + start_pos.x, top),
                            point(bounds.left() + end_pos.x, top + line_height),
                        ),
                        selection_color,
                    ));
                } else {
                    // Selection crosses wrap rows: highlight to the right edge
                    // on the first row, every full row in between, and up to
                    // the end position on the last row.
                    let start_top = line_top + start_pos.y;
                    selections.push(fill(
                        Bounds::from_corners(
                            point(bounds.left() + start_pos.x, start_top),
                            point(bounds.right(), start_top + line_height),
                        ),
                        selection_color,
                    ));
                    let end_top = line_top + end_pos.y;
                    if end_top > start_top + line_height {
                        selections.push(fill(
                            Bounds::from_corners(
                                point(bounds.left(), start_top + line_height),
                                point(bounds.right(), end_top),
                            ),
                            selection_color,
                        ));
                    }
                    selections.push(fill(
                        Bounds::from_corners(
                            point(bounds.left(), end_top),
                            point(bounds.left() + end_pos.x, end_top + line_height),
                        ),
                        selection_color,
                    ));
                }
            }
        }

        PrepaintState {
            lines,
            line_offsets,
            line_heights,
            cursors,
            selections,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.app.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.app.clone()),
            cx,
        );
        let line_height = window.line_height();
        // Only paint lines that intersect the visible clip region (the scroll
        // container's content mask). `WrappedLine::paint` walks every glyph on
        // the CPU even when the glyphs are ultimately masked out, so painting
        // the off-screen lines of a large document costs a full walk of the
        // whole text every frame; skipping them makes paint O(visible lines).
        let visible = window.content_mask().bounds;
        let mut cumulative_y = px(0.);
        for (index, line) in prepaint.lines.iter().enumerate() {
            let top = bounds.top() + cumulative_y;
            let height = prepaint
                .line_heights
                .get(index)
                .copied()
                .unwrap_or(line_height);
            cumulative_y += height;
            if top > visible.bottom() {
                break;
            }
            if top + height < visible.top() {
                continue;
            }
            line.paint(
                point(bounds.left(), top),
                line_height,
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();
        }
        for selection in prepaint.selections.drain(..) {
            window.paint_quad(selection);
        }
        if focus_handle.is_focused(window) {
            for cursor in prepaint.cursors.drain(..) {
                window.paint_quad(cursor);
            }
        }
        let lines = std::mem::take(&mut prepaint.lines);
        let line_offsets = std::mem::take(&mut prepaint.line_offsets);
        let line_heights = std::mem::take(&mut prepaint.line_heights);
        self.app.update(cx, |app, _cx| {
            let tab = app.active_tab_mut();
            tab.last_lines = lines;
            tab.line_offsets = line_offsets;
            tab.line_heights = line_heights;
            tab.last_bounds = Some(bounds);
            tab.line_height = line_height;
        });
    }
}
