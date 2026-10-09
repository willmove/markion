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
pub(super) struct AiInput {
    pub focus: FocusHandle,
    pub field: SearchFieldState,
    pub multiline: bool,
    pub masked: bool,
    pub(super) undo: Vec<SearchFieldState>,
    redo: Vec<SearchFieldState>,
    composition: Option<SearchFieldState>,
    lines: Vec<(usize, ShapedLine)>,
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
        cx.notify();
    }
    fn checkpoint(&mut self) {
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
        cx.notify();
    }
    fn vertical(&mut self, delta: isize) {
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = cx.entity();
        let paint_input = input.clone();
        let height = px(self.field.buffer.lines().count().max(1) as f32 * 20. + 20.);
        let view = div()
            .id("ai-input")
            .key_context("AiInput")
            .track_focus(&self.focus)
            .tab_index(0)
            .w_full()
            .max_h(px(if self.multiline { 140. } else { 42. }))
            .min_h(px(36.))
            .overflow_scroll()
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
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputRight, _, cx| {
                s.field.move_caret(SearchCaretMove::Right);
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputSelectLeft, _, cx| {
                s.field.move_caret(SearchCaretMove::SelectLeft);
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputSelectRight, _, cx| {
                s.field.move_caret(SearchCaretMove::SelectRight);
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputSelectAll, _, cx| {
                s.field.move_caret(SearchCaretMove::SelectAll);
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputHome, _, cx| {
                s.field.move_caret(SearchCaretMove::Home);
                cx.stop_propagation();
                cx.notify();
            }))
            .on_action(cx.listener(|s, _: &InputEnd, _, cx| {
                s.field.move_caret(SearchCaretMove::End);
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
                if event.keystroke.key != "tab" && event.keystroke.key != "escape" {
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
                        let s = input.read(cx);
                        let style = window.text_style();
                        let run = TextRun {
                            len: 0,
                            font: style.font(),
                            color: style.color,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        let mut offset = 0;
                        let mut lines = Vec::new();
                        for text in s.field.buffer.split('\n') {
                            let display = if s.masked {
                                "*".repeat(text.len())
                            } else {
                                text.into()
                            };
                            let len = display.len();
                            lines.push((
                                offset,
                                window.text_system().shape_line(
                                    display.into(),
                                    px(13.),
                                    &[TextRun { len, ..run.clone() }],
                                    None,
                                ),
                            ));
                            offset += text.len() + 1;
                        }
                        (bounds, lines)
                    },
                    move |_, (bounds, lines), window, cx| {
                        let s = paint_input.read(cx);
                        let focus = s.focus.clone();
                        let selection = s.field.selection();
                        let cursor = s.field.cursor;
                        let marked = s.field.marked_range.clone();
                        let color = window.text_style().color;
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
                            if focus.is_focused(window) && cursor >= *start && cursor <= end {
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
