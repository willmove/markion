//! An independent input entity: platform composition and Undo never visit the document.
use super::*;
use gpui::{EventEmitter, ShapedLine};
actions!(
    ai_input,
    [
        InputBackspace,
        InputDelete,
        InputLeft,
        InputRight,
        InputSelectLeft,
        InputSelectRight,
        InputSelectAll,
        InputHome,
        InputEnd,
        InputUp,
        InputDown,
        InputPaste,
        InputCopy,
        InputCut,
        InputUndo,
        InputRedo,
        InputEnter,
        InputNewline
    ]
);
pub(super) struct Submit;
fn is_text_key(event: &KeyDownEvent) -> bool {
    let modifiers = event.keystroke.modifiers;
    event.prefer_character_input
        || (!(modifiers.control || modifiers.alt || modifiers.platform || modifiers.function)
            && (event.keystroke.key_char.is_some() || event.keystroke.key == "space"))
}
pub(super) struct AiInput {
    pub focus: FocusHandle,
    pub field: SearchFieldState,
    pub multiline: bool,
    pub masked: bool,
    pub(super) undo: Vec<SearchFieldState>,
    redo: Vec<SearchFieldState>,
    composition: Option<SearchFieldState>,
    lines: Vec<(usize, ShapedLine)>,
    layout_key: Option<(String, Pixels, TextRun)>,
    reveal_caret: bool,
    bounds: Option<Bounds<Pixels>>,
    selecting: bool,
    scroll: ScrollHandle,
}
impl EventEmitter<Submit> for AiInput {}
impl Focusable for AiInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl AiInput {
    pub fn new(
        text: impl Into<String>,
        multiline: bool,
        masked: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            focus: cx.focus_handle(),
            field: SearchFieldState::new(text),
            multiline,
            masked,
            undo: Vec::new(),
            redo: Vec::new(),
            composition: None,
            lines: Vec::new(),
            layout_key: None,
            reveal_caret: true,
            bounds: None,
            selecting: false,
            scroll: ScrollHandle::new(),
        }
    }
    pub fn text(&self) -> &str {
        &self.field.buffer
    }
    pub fn set(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.field.set_text(text);
        self.undo.clear();
        self.redo.clear();
        self.composition = None;
        self.reveal_caret = true;
        cx.notify();
    }
    fn checkpoint(&mut self) {
        self.reveal_caret = true;
        if self.undo.len() >= 100 {
            self.undo.remove(0);
        }
        self.undo.push(self.field.clone());
        self.redo.clear();
    }
    pub(super) fn replace(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        marked: bool,
        selection: Option<Range<usize>>,
        cx: &mut Context<Self>,
    ) {
        let range = range
            .map(|r| self.field.range_from_utf16(r))
            .or_else(|| self.field.marked_range.clone())
            .unwrap_or_else(|| self.field.selection());
        let text = if self.multiline {
            text.to_owned()
        } else {
            text.replace(['\r', '\n'], " ")
        };
        if self.field.buffer.len() - range.len() + text.len() > 64 * 1024 {
            return;
        }
        if marked {
            if self.composition.is_none() {
                self.composition = Some(self.field.clone());
            }
        } else if let Some(original) = self.composition.take() {
            self.undo.push(original);
            self.redo.clear();
        } else {
            self.checkpoint();
        }
        self.field.buffer.replace_range(range.clone(), &text);
        let end = range.start + text.len();
        self.field.cursor = end;
        self.field.anchor = end;
        self.field.marked_range = marked.then_some(range.start..end);
        if let Some(selected) = selection {
            let temporary = SearchFieldState::new(text);
            let selected = temporary.range_from_utf16(selected);
            self.field.anchor = range.start + selected.start;
            self.field.cursor = range.start + selected.end;
        }
        self.reveal_caret = true;
        cx.notify();
    }
    fn vertical(&mut self, delta: isize) {
        if !self.lines.is_empty()
            && self
                .layout_key
                .as_ref()
                .is_some_and(|k| k.0 == self.field.buffer)
        {
            let row = self.caret_row();
            let next = (row as isize + delta).clamp(0, self.lines.len() as isize - 1) as usize;
            let (start, line) = &self.lines[row];
            let x = line.x_for_index(self.field.cursor.saturating_sub(*start).min(line.len()));
            let (start, line) = &self.lines[next];
            self.field.cursor =
                clamp_search_boundary(&self.field.buffer, *start + line.closest_index_for_x(x));
            self.field.anchor = self.field.cursor;
            self.reveal_caret = true;
            return;
        }
        let cursor = self.field.cursor;
        let start = self.field.buffer[..cursor].rfind('\n').map_or(0, |i| i + 1);
        let column = self.field.buffer[start..cursor].chars().count();
        let next = if delta < 0 {
            if start == 0 {
                return;
            }
            let end = start - 1;
            self.field.buffer[..end].rfind('\n').map_or(0, |i| i + 1)
        } else {
            let Some(end) = self.field.buffer[cursor..].find('\n') else {
                return;
            };
            cursor + end + 1
        };
        let rest = self.field.buffer[next..].split('\n').next().unwrap_or("");
        let offset = rest
            .char_indices()
            .nth(column)
            .map_or(rest.len(), |(i, _)| i);
        self.field.cursor = next + offset;
        self.field.anchor = self.field.cursor;
        self.reveal_caret = true;
    }
    fn caret_row(&self) -> usize {
        self.lines
            .iter()
            .rposition(|(start, _)| *start <= self.field.cursor)
            .unwrap_or(0)
    }
    fn shape_rows(&mut self, width: Pixels, window: &Window) {
        let style = window.text_style();
        let run = TextRun {
            len: self.field.buffer.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let key = (self.field.buffer.clone(), width, run.clone());
        if self.layout_key.as_ref() == Some(&key) {
            return;
        }
        self.lines.clear();
        let mut offset = 0;
        for text in self.field.buffer.split('\n') {
            let first_row = self.lines.len();
            if self.multiline && !text.is_empty() {
                if let Ok(wrapped) = window.text_system().shape_text(
                    text.to_owned().into(),
                    px(13.),
                    &[TextRun {
                        len: text.len(),
                        ..run.clone()
                    }],
                    Some(width.max(px(1.))),
                    None,
                ) {
                    for line in wrapped {
                        let mut starts = vec![0];
                        starts.extend(
                            line.wrap_boundaries
                                .iter()
                                .map(|b| line.runs()[b.run_ix].glyphs[b.glyph_ix].index),
                        );
                        starts.push(text.len());
                        for range in starts.windows(2) {
                            let segment = &text[range[0]..range[1]];
                            self.lines.push((
                                offset + range[0],
                                window.text_system().shape_line(
                                    segment.to_owned().into(),
                                    px(13.),
                                    &[TextRun {
                                        len: segment.len(),
                                        ..run.clone()
                                    }],
                                    None,
                                ),
                            ));
                        }
                    }
                }
            } else {
                let display: String = if self.masked {
                    "*".repeat(text.len())
                } else {
                    text.into()
                };
                let len = display.len();
                self.lines.push((
                    offset,
                    window.text_system().shape_line(
                        display.into(),
                        px(13.),
                        &[TextRun { len, ..run.clone() }],
                        None,
                    ),
                ));
            }
            if self.lines.len() == first_row {
                self.lines.push((
                    offset,
                    window.text_system().shape_line(
                        text.to_owned().into(),
                        px(13.),
                        &[TextRun {
                            len: text.len(),
                            ..run.clone()
                        }],
                        None,
                    ),
                ));
            }
            offset += text.len() + 1;
        }
        self.layout_key = Some(key);
        self.reveal_caret = true;
    }
    fn index(&self, position: Point<Pixels>) -> usize {
        let Some(bounds) = self.bounds else {
            return self.field.cursor;
        };
        let row = ((f32::from(position.y - bounds.top()) / 20.).floor().max(0.) as usize)
            .min(self.lines.len().saturating_sub(1));
        let Some((start, line)) = self.lines.get(row) else {
            return 0;
        };
        let index = start + line.closest_index_for_x(position.x - bounds.left());
        clamp_search_boundary(&self.field.buffer, index.min(self.field.buffer.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn direct_printable_keystrokes_reach_input(cx: &mut TestAppContext) {
        // Windows represents Space without key_char; simulated IME normally fills it in.
        let mut space = KeyDownEvent {
            keystroke: gpui::Keystroke::parse("space").unwrap(),
            is_held: false,
            prefer_character_input: false,
        };
        assert!(space.keystroke.key_char.is_none());
        assert!(is_text_key(&space));
        space.keystroke.modifiers.control = true;
        assert!(!is_text_key(&space));
        space.prefer_character_input = true;
        assert!(is_text_key(&space));
        for multiline in [false, true] {
            let (input, cx) = cx.add_window_view(|_, cx| AiInput::new("", multiline, false, cx));
            cx.update(|window, cx| {
                bind(cx);
                window.activate_window();
                window.focus(&input.read(cx).focus);
            });
            cx.simulate_keystrokes("h e l l o space shift-w o r l d . 1");
            input.update(cx, |input, _| {
                assert_eq!(input.text(), "hello World.1");
                assert!(input.field.marked_range.is_none());
            });
        }
    }

    #[gpui::test]
    fn wrapped_rows_preserve_utf8_navigation_hit_testing_and_ime(cx: &mut TestAppContext) {
        let text = "中文😀连续输入没有手动换行。Long prompt with several words.\n下一行😀";
        let (input, cx) = cx.add_window_view(|_, cx| AiInput::new(text, true, false, cx));
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                input.shape_rows(px(80.), window);
                let narrow_rows = input.lines.len();
                assert!(narrow_rows > 2);
                assert_eq!(input.text(), text);
                assert!(input.undo.is_empty());
                input.bounds = Some(Bounds::new(
                    point(px(10.), px(10.)),
                    size(px(80.), px(200.)),
                ));
                for (row, (start, line)) in input.lines.iter().enumerate() {
                    assert!(text.is_char_boundary(*start));
                    assert!(text.is_char_boundary(*start + line.len()));
                    assert_eq!(
                        input.index(point(px(10.), px(12. + row as f32 * 20.))),
                        *start
                    );
                }
                input.field.cursor = input.lines[1].0;
                input.field.anchor = input.field.cursor;
                input.vertical(-1);
                assert_eq!(input.field.cursor, 0);
                input.vertical(1);
                assert_eq!(input.field.cursor, input.lines[1].0);
                let offset = input.field.byte_to_utf16(input.field.cursor);
                let bounds = input
                    .bounds_for_range(offset..offset, input.bounds.unwrap(), window, cx)
                    .unwrap();
                assert_eq!(bounds.top(), px(30.));
                input.shape_rows(px(240.), window);
                assert!(input.lines.len() < narrow_rows);
                assert_eq!(input.text(), text);
                input.replace_and_mark_text_in_range(
                    Some(offset..offset),
                    "输入😀",
                    Some(4..4),
                    window,
                    cx,
                );
                assert_eq!(
                    input.marked_text_range(window, cx),
                    Some(offset..offset + 4)
                );
                input.shape_rows(px(80.), window);
                let cursor = input.field.byte_to_utf16(input.field.cursor);
                let bounds = input
                    .bounds_for_range(cursor..cursor, input.bounds.unwrap(), window, cx)
                    .unwrap();
                let hit = input
                    .character_index_for_point(bounds.origin, window, cx)
                    .unwrap();
                assert_eq!(hit, cursor);
                input.unmark_text(window, cx);
                assert_eq!(input.undo.len(), 1);
                input.field = input.undo.pop().unwrap();
                assert_eq!(input.text(), text);
            });
        });
    }
}
pub(super) fn bind(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", InputBackspace, Some("AiInput")),
        KeyBinding::new("delete", InputDelete, Some("AiInput")),
        KeyBinding::new("left", InputLeft, Some("AiInput")),
        KeyBinding::new("right", InputRight, Some("AiInput")),
        KeyBinding::new("shift-left", InputSelectLeft, Some("AiInput")),
        KeyBinding::new("shift-right", InputSelectRight, Some("AiInput")),
        KeyBinding::new("secondary-a", InputSelectAll, Some("AiInput")),
        KeyBinding::new("home", InputHome, Some("AiInput")),
        KeyBinding::new("end", InputEnd, Some("AiInput")),
        KeyBinding::new("up", InputUp, Some("AiInput")),
        KeyBinding::new("down", InputDown, Some("AiInput")),
        KeyBinding::new("secondary-v", InputPaste, Some("AiInput")),
        KeyBinding::new("secondary-c", InputCopy, Some("AiInput")),
        KeyBinding::new("secondary-x", InputCut, Some("AiInput")),
        KeyBinding::new("secondary-z", InputUndo, Some("AiInput")),
        KeyBinding::new("secondary-shift-z", InputRedo, Some("AiInput")),
        KeyBinding::new("secondary-y", InputRedo, Some("AiInput")),
        KeyBinding::new("enter", InputEnter, Some("AiInput")),
        KeyBinding::new("shift-enter", InputNewline, Some("AiInput")),
    ]);
}
impl EntityInputHandler for AiInput {
    fn text_for_range(
        &mut self,
        r: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.field.range_from_utf16(r);
        *actual = Some(self.field.byte_to_utf16(r.start)..self.field.byte_to_utf16(r.end));
        Some(self.field.buffer[r].into())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let r = self.field.selection();
        Some(UTF16Selection {
            range: self.field.byte_to_utf16(r.start)..self.field.byte_to_utf16(r.end),
            reversed: self.field.cursor < self.field.anchor,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.field
            .marked_range
            .clone()
            .map(|r| self.field.byte_to_utf16(r.start)..self.field.byte_to_utf16(r.end))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.field.marked_range = None;
        if let Some(original) = self.composition.take() {
            self.undo.push(original);
            self.redo.clear();
        }
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace(r, text, false, None, cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace(r, text, true, selected, cx);
    }
    fn bounds_for_range(
        &mut self,
        r: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let offset = self.field.utf16_to_byte(r.start);
        let bounds = self.bounds?;
        let (row, (start, line)) = self
            .lines
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (start, _))| *start <= offset)?;
        Some(Bounds::new(
            point(
                bounds.left() + line.x_for_index(offset - start),
                bounds.top() + px(row as f32 * 20.),
            ),
            size(px(2.), px(20.)),
        ))
    }
    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.field.byte_to_utf16(self.index(p)))
    }
}
impl Render for AiInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width = self.bounds.map_or(px(280.), |b| b.size.width);
        self.shape_rows(width, window);
        if self.reveal_caret {
            let row = self.caret_row();
            let y = px(row as f32 * 20.);
            let viewport = self.scroll.bounds().size.height.max(px(36.));
            let mut offset = self.scroll.offset();
            if y + offset.y < px(0.) {
                offset.y = -y;
            }
            if y + px(20.) + offset.y > viewport {
                offset.y = viewport - y - px(20.);
            }
            if self.multiline {
                offset.x = px(0.);
            } else if let Some((start, line)) = self.lines.get(row) {
                let x = line.x_for_index(self.field.cursor - start);
                let viewport = self.scroll.bounds().size.width.max(px(36.));
                if x + offset.x < px(0.) {
                    offset.x = -x;
                }
                if x + px(4.) + offset.x > viewport {
                    offset.x = viewport - x - px(4.);
                }
            }
            self.scroll.set_offset(offset);
            self.reveal_caret = false;
        }
        let input = cx.entity();
        let paint_input = input.clone();
        let height = px(self.lines.len().max(1) as f32 * 20. + 12.);
        let view = div()
            .id("ai-input")
            .key_context("AiInput")
            .track_focus(&self.focus)
            .tab_index(0)
            .w_full()
            .max_h(px(if self.multiline { 140. } else { 42. }))
            .min_h(px(36.))
            .overflow_y_scroll()
            .overflow_x_hidden()
            .when(!self.multiline, |d| d.overflow_x_scroll())
            .track_scroll(&self.scroll)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(|s, _: &InputBackspace, _, cx| {
                s.checkpoint();
                s.field.backspace();
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputDelete, _, cx| {
                s.checkpoint();
                s.field.delete_forward();
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputLeft, _, cx| {
                s.field.move_caret(SearchCaretMove::Left);
                s.reveal_caret = true;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputRight, _, cx| {
                s.field.move_caret(SearchCaretMove::Right);
                s.reveal_caret = true;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputSelectLeft, _, cx| {
                s.field.move_caret(SearchCaretMove::SelectLeft);
                s.reveal_caret = true;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputSelectRight, _, cx| {
                s.field.move_caret(SearchCaretMove::SelectRight);
                s.reveal_caret = true;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputSelectAll, _, cx| {
                s.field.move_caret(SearchCaretMove::SelectAll);
                s.reveal_caret = true;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputHome, _, cx| {
                s.field.move_caret(SearchCaretMove::Home);
                s.reveal_caret = true;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputEnd, _, cx| {
                s.field.move_caret(SearchCaretMove::End);
                s.reveal_caret = true;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputUp, _, cx| {
                s.vertical(-1);
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputDown, _, cx| {
                s.vertical(1);
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputPaste, _, cx| {
                if let Some(t) = cx.read_from_clipboard().and_then(|c| c.text()) {
                    s.replace(None, &t, false, None, cx);
                }
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|s, _: &InputCopy, _, cx| {
                if !s.masked {
                    if let Some(t) = s.field.selected_text() {
                        cx.write_to_clipboard(ClipboardItem::new_string(t.into()));
                    }
                }
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|s, _: &InputCut, _, cx| {
                if !s.masked {
                    if let Some(t) = s.field.selected_text() {
                        cx.write_to_clipboard(ClipboardItem::new_string(t.into()));
                    }
                    s.replace(None, "", false, None, cx);
                }
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|s, _: &InputUndo, _, cx| {
                if s.field.marked_range.is_none() {
                    if let Some(old) = s.undo.pop() {
                        s.redo.push(s.field.clone());
                        s.field = old;
                        s.reveal_caret = true;
                        cx.notify();
                    }
                }
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|s, _: &InputRedo, _, cx| {
                if s.field.marked_range.is_none() {
                    if let Some(old) = s.redo.pop() {
                        s.undo.push(s.field.clone());
                        s.field = old;
                        s.reveal_caret = true;
                        cx.notify();
                    }
                }
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|s, _: &InputEnter, _, cx| {
                if s.field.marked_range.is_none() && s.multiline {
                    cx.emit(Submit);
                }
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|s, _: &InputNewline, _, cx| {
                if s.field.marked_range.is_none() && s.multiline {
                    s.replace(None, "\n", false, None, cx);
                }
                cx.stop_propagation();
            }))
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                // Printable keys must reach the platform text handler (WM_CHAR on Windows).
                if !is_text_key(event)
                    && event.keystroke.key != "tab"
                    && event.keystroke.key != "escape"
                {
                    cx.stop_propagation();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|s, event: &MouseDownEvent, w, cx| {
                    w.focus(&s.focus);
                    s.selecting = true;
                    s.field.cursor = s.index(event.position);
                    if !event.modifiers.shift {
                        s.field.anchor = s.field.cursor;
                    }
                    cx.stop_propagation();
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|s, event: &MouseMoveEvent, _, cx| {
                if s.selecting && event.dragging() {
                    s.field.cursor = s.index(event.position);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|s, _, _, _| s.selecting = false),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|s, _, _, _| s.selecting = false),
            )
            .child(
                canvas(
                    move |bounds, window, cx| {
                        let lines = input.update(cx, |s, cx| {
                            let previous = s.lines.len();
                            let previous_width = s.layout_key.as_ref().map(|key| key.1);
                            s.shape_rows(bounds.size.width, window);
                            if previous != s.lines.len()
                                || previous_width != Some(bounds.size.width)
                            {
                                cx.notify();
                            }
                            s.lines.clone()
                        });
                        (bounds, lines)
                    },
                    move |_, (bounds, lines), window, cx| {
                        let s = paint_input.read(cx);
                        let focus = s.focus.clone();
                        let selection = s.field.selection();
                        let cursor = s.field.cursor;
                        let marked = s.field.marked_range.clone();
                        let color = window.text_style().color;
                        let caret_row = s.caret_row();
                        window.handle_input(
                            &focus,
                            ElementInputHandler::new(bounds, paint_input.clone()),
                            cx,
                        );
                        for (row, (start, line)) in lines.iter().enumerate() {
                            let origin = point(bounds.left(), bounds.top() + px(row as f32 * 20.));
                            let end = *start + line.len();
                            let a = selection.start.max(*start).min(end);
                            let b = selection.end.max(*start).min(end);
                            if a < b {
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(origin.x + line.x_for_index(a - start), origin.y),
                                        size(
                                            line.x_for_index(b - start)
                                                - line.x_for_index(a - start),
                                            px(20.),
                                        ),
                                    ),
                                    rgba(0x5588cc55),
                                ));
                            }
                            let _ = line.paint(origin, px(20.), window, cx);
                            if focus.is_focused(window) && row == caret_row {
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(
                                            origin.x + line.x_for_index(cursor - start),
                                            origin.y,
                                        ),
                                        size(px(1.5), px(20.)),
                                    ),
                                    color,
                                ));
                            }
                            if let Some(r) = &marked {
                                let a = r.start.max(*start).min(end);
                                let b = r.end.max(*start).min(end);
                                if a < b {
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(
                                                origin.x + line.x_for_index(a - start),
                                                origin.y + px(18.),
                                            ),
                                            size(
                                                line.x_for_index(b - start)
                                                    - line.x_for_index(a - start),
                                                px(1.),
                                            ),
                                        ),
                                        color,
                                    ));
                                }
                            }
                        }
                        paint_input.update(cx, |s, _| {
                            s.bounds = Some(bounds);
                            s.lines = lines;
                        });
                    },
                )
                .w_full()
                .when(!self.multiline, |d| {
                    d.min_w(
                        self.lines
                            .first()
                            .map_or(px(0.), |(_, line)| line.width + px(4.)),
                    )
                })
                .h(height),
            );
        macro_rules! block {($view:expr,$($action:ty),+)=>{$view$(.on_action(cx.listener(|_,_:&$action,_,cx|cx.stop_propagation())))+};}
        let view = block!(
            view,
            Bold,
            Italic,
            InlineCode,
            InsertLink,
            InsertImage,
            InsertImageFile,
            InsertImageUrl,
            Paragraph,
            Heading1,
            Heading2,
            Heading3,
            Heading4,
            Heading5,
            Heading6,
            UnorderedList,
            OrderedList,
            TaskList,
            BlockQuote,
            CodeFence,
            FormatTable,
            TableAddRow,
            TableDeleteRow,
            TableMoveRowUp,
            TableMoveRowDown,
            TableAddColumn,
            TableDeleteColumn
        );
        macro_rules! forward {($view:expr,$($from:ty=>$to:expr),+)=>{$view$(.on_action(cx.listener(|_,_:&$from,w,cx|{w.dispatch_action(Box::new($to),cx);cx.stop_propagation();})))+};}
        let view = view
            .on_action(cx.listener(|_, _: &Indent, w, cx| {
                w.focus_next();
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|_, _: &Outdent, w, cx| {
                w.focus_prev();
                cx.stop_propagation();
            }));
        forward!(view,Undo=>InputUndo,Redo=>InputRedo,Paste=>InputPaste,PastePlainText=>InputPaste,Copy=>InputCopy,Cut=>InputCut,SelectAll=>InputSelectAll,Backspace=>InputBackspace,Delete=>InputDelete,Left=>InputLeft,Right=>InputRight,Up=>InputUp,Down=>InputDown,SelectLeft=>InputSelectLeft,SelectRight=>InputSelectRight,Home=>InputHome,End=>InputEnd,InsertNewline=>InputNewline,SearchPreviousOrNewline=>InputNewline)
    }
}
