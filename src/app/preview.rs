use super::*;

pub(super) fn read_mode_preview_is_constrained(
    view_mode: ViewMode,
    preview_adaptive_width: bool,
) -> bool {
    matches!(view_mode, ViewMode::Read) && !preview_adaptive_width
}

pub(super) fn view_mode_status_message(view_mode: ViewMode) -> Msg {
    match view_mode {
        ViewMode::Edit => Msg::StatusEditMode,
        ViewMode::VisualEdit => Msg::StatusVisualEditMode,
        ViewMode::Split => Msg::StatusSplitPreviewMode,
        ViewMode::Read => Msg::StatusReadMode,
    }
}

pub(super) fn assign_view_mode(current: &mut ViewMode, target: ViewMode) {
    *current = target;
}

pub(super) fn view_mode_pane_widths(view_mode: ViewMode, split_ratio: f32) -> (f32, f32) {
    match view_mode {
        ViewMode::Edit | ViewMode::VisualEdit => (1.0, 0.0),
        ViewMode::Split => (split_ratio, 1.0 - split_ratio),
        ViewMode::Read => (0.0, 1.0),
    }
}

/// Whether proportional scroll-sync should couple the two panes this frame.
/// Only in Split Preview (the sole mode where both panes are visible) and only
/// when the preference is on.
pub(super) fn sync_scroll_is_active(view_mode: ViewMode, sync_scroll: bool) -> bool {
    matches!(view_mode, ViewMode::Split) && sync_scroll
}

/// Scroll fraction in `[0,1]` for a pane, given its current scroll offset
/// (positive pixels from the top) and its maximum scrollable offset. Returns
/// `0.0` when the pane has no scrollable range (`max <= 1`), so a pane that
/// fits its viewport never drives the other pane.
pub(super) fn sync_fraction(offset: f32, max: f32) -> f32 {
    if max <= 1. {
        return 0.;
    }
    (offset / max).clamp(0., 1.)
}

/// Sync coupling converges within an epsilon; comparing fractions below this
/// threshold avoids re-writing the non-driving pane every frame (and the
/// resulting sub-pixel fight with the user's own scroll).
pub(super) const SYNC_SCROLL_EPSILON: f32 = 0.001;

/// Clamp a byte offset to a UTF-8 char boundary within `run_text`.
pub(super) fn clamp_preview_offset(run_text: &str, offset: usize) -> usize {
    let mut offset = offset.min(run_text.len());
    while offset < run_text.len() && !run_text.is_char_boundary(offset) {
        offset += 1;
    }
    if offset > run_text.len() {
        return run_text.len();
    }
    while offset > 0 && !run_text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

/// Normalize a preview selection range against `run_text`, clamping to UTF-8
/// char boundaries and ensuring `start <= end`.
pub(super) fn normalize_preview_selection_range(
    run_text: &str,
    range: Range<usize>,
) -> Range<usize> {
    let start = clamp_preview_offset(run_text, range.start.min(range.end));
    let end = clamp_preview_offset(run_text, range.start.max(range.end));
    start..end
}

/// Plain text of a single selectable run inside a preview block.
pub(super) fn preview_run_plain_text(
    block: &PreviewBlock,
    run_id: PreviewTextRunId,
) -> Option<String> {
    match (block, run_id) {
        (
            PreviewBlock::Heading { text, .. }
            | PreviewBlock::Paragraph { text, .. }
            | PreviewBlock::ListItem { text, .. }
            | PreviewBlock::BlockQuote { text, .. },
            PreviewTextRunId::Body,
        ) => Some(text.text.clone()),
        (PreviewBlock::CodeBlock { code, .. }, PreviewTextRunId::CodeBody) => Some(code.clone()),
        (PreviewBlock::CodeBlock { code, .. }, PreviewTextRunId::CodeLine(line_index)) => {
            code.lines().nth(line_index).map(|line| line.to_string())
        }
        (PreviewBlock::MathBlock { latex, .. }, PreviewTextRunId::MathLatex) => Some(latex.clone()),
        (PreviewBlock::MathBlock { latex, .. }, PreviewTextRunId::MathRendered) => {
            Some(render_math(latex, true).text)
        }
        (PreviewBlock::Html { html, .. }, PreviewTextRunId::HtmlText) => {
            Some(html_preview_plain_text(html))
        }
        (PreviewBlock::Image { alt, url, .. }, PreviewTextRunId::ImageCaption) => {
            let caption = if alt.is_empty() {
                url.as_str()
            } else {
                alt.as_str()
            };
            Some(format!("Image: {caption}"))
        }
        (PreviewBlock::Image { url, title, .. }, PreviewTextRunId::ImageMeta) => {
            Some(if title.as_deref().unwrap_or("").is_empty() {
                url.clone()
            } else {
                format!("{url} - {}", title.as_deref().unwrap_or(""))
            })
        }
        (PreviewBlock::Table { rows, .. }, PreviewTextRunId::TableCell { row, col }) => {
            rows.get(row).and_then(|r| r.get(col)).cloned()
        }
        _ => None,
    }
}

/// Document-order list of selectable runs for a preview block.
pub(super) fn preview_block_runs(block: &PreviewBlock) -> Vec<PreviewTextRunId> {
    match block {
        PreviewBlock::Heading { .. }
        | PreviewBlock::Paragraph { .. }
        | PreviewBlock::ListItem { .. }
        | PreviewBlock::BlockQuote { .. } => vec![PreviewTextRunId::Body],
        PreviewBlock::CodeBlock { .. } => vec![PreviewTextRunId::CodeBody],
        PreviewBlock::MathBlock { .. } => {
            vec![PreviewTextRunId::MathRendered, PreviewTextRunId::MathLatex]
        }
        PreviewBlock::Html { html, .. } => (!html_preview_plain_text(html).is_empty())
            .then_some(PreviewTextRunId::HtmlText)
            .into_iter()
            .collect(),
        PreviewBlock::Image { .. } => {
            vec![PreviewTextRunId::ImageCaption, PreviewTextRunId::ImageMeta]
        }
        PreviewBlock::Table { rows, .. } => rows
            .iter()
            .enumerate()
            .flat_map(|(row, cols)| {
                (0..cols.len()).map(move |col| PreviewTextRunId::TableCell { row, col })
            })
            .collect(),
        PreviewBlock::Rule { .. } => Vec::new(),
    }
}

/// Byte range to highlight inside `run_id` for a free-range selection, if any.
pub(super) fn preview_run_highlight_range(
    selection: &PreviewSelection,
    block_index: usize,
    run_id: PreviewTextRunId,
    run_text: &str,
) -> Option<Range<usize>> {
    let run_len = run_text.len();
    let (start, end) = selection.ordered_carets();
    let caret = PreviewCaret {
        block_index,
        run_id,
        offset: 0,
    };
    let caret_end = PreviewCaret {
        block_index,
        run_id,
        offset: run_len,
    };
    // Run entirely before or after the selection.
    if caret_end.cmp_doc_order(start) != std::cmp::Ordering::Greater
        || caret.cmp_doc_order(end) != std::cmp::Ordering::Less
    {
        return None;
    }
    let range_start = if start.block_index == block_index && start.run_id == run_id {
        start.offset.min(run_len)
    } else {
        0
    };
    let range_end = if end.block_index == block_index && end.run_id == run_id {
        end.offset.min(run_len)
    } else {
        run_len
    };
    let range = normalize_preview_selection_range(run_text, range_start..range_end);
    if range.is_empty() { None } else { Some(range) }
}

/// Plain text for a free-range preview selection across contiguous runs.
pub(super) fn preview_selection_plain_text(
    selection: &PreviewSelection,
    blocks: &[PreviewBlock],
) -> Option<String> {
    if selection.is_empty_carets() {
        return None;
    }
    let (start, end) = selection.ordered_carets();
    if start.block_index >= blocks.len() || end.block_index >= blocks.len() {
        return None;
    }
    let mut parts = Vec::new();
    for block_index in start.block_index..=end.block_index {
        let block = &blocks[block_index];
        let runs = preview_block_runs(block);
        for run_id in runs {
            let Some(text) = preview_run_plain_text(block, run_id) else {
                continue;
            };
            let run_start = PreviewCaret {
                block_index,
                run_id,
                offset: 0,
            };
            let run_end = PreviewCaret {
                block_index,
                run_id,
                offset: text.len(),
            };
            if run_end.cmp_doc_order(start) != std::cmp::Ordering::Greater
                || run_start.cmp_doc_order(end) != std::cmp::Ordering::Less
            {
                continue;
            }
            let from = if start.block_index == block_index && start.run_id == run_id {
                clamp_preview_offset(&text, start.offset)
            } else {
                0
            };
            let to = if end.block_index == block_index && end.run_id == run_id {
                clamp_preview_offset(&text, end.offset)
            } else {
                text.len()
            };
            if from < to {
                parts.push(text[from..to].to_string());
            }
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Whether Copy should prefer the preview selection over the source editor.
pub(super) fn preview_selection_takes_copy_precedence(
    preview: Option<&PreviewSelection>,
    blocks: &[PreviewBlock],
) -> bool {
    preview.is_some_and(|selection| preview_selection_plain_text(selection, blocks).is_some())
}

/// Drop a preview selection when either caret's block index is out of range.
pub(super) fn invalidate_preview_selection_if_stale(
    selection: Option<PreviewSelection>,
    block_count: usize,
) -> Option<PreviewSelection> {
    match selection {
        Some(sel) if sel.anchor.block_index < block_count && sel.head.block_index < block_count => {
            Some(sel)
        }
        _ => None,
    }
}

/// Source Markdown for the blocks covered by a preview selection.
pub(super) fn preview_selection_markdown(
    selection: &PreviewSelection,
    blocks: &[PreviewBlock],
    document: &str,
) -> Option<String> {
    if selection.is_empty_carets() {
        return None;
    }
    let (start, end) = selection.ordered_carets();
    if start.block_index >= blocks.len() || end.block_index >= blocks.len() {
        return None;
    }
    let mut slices = Vec::new();
    for block_index in start.block_index..=end.block_index {
        let range = preview_block_source_range(&blocks[block_index])?;
        if range.start >= document.len() {
            continue;
        }
        let end_byte = range.end.min(document.len());
        let start_byte = range.start.min(end_byte);
        if start_byte < end_byte {
            slices.push(document[start_byte..end_byte].trim_end().to_string());
        }
    }
    if slices.is_empty() {
        None
    } else {
        Some(slices.join("\n\n"))
    }
}

pub(super) fn preview_block_source_range(block: &PreviewBlock) -> Option<Range<usize>> {
    Some(block.source_range().clone())
}

/// Preview color accents shared across themes. Block chrome colors stay in
/// line with the previous hardcoded preview styling.
const PREVIEW_LINK_COLOR: u32 = 0x2563eb;
const PREVIEW_SELECTION_COLOR: u32 = 0x2563eb30;
const PREVIEW_INLINE_CODE_COLOR: u32 = 0xdb2777;
const PREVIEW_INLINE_CODE_BG: u32 = 0x64748b26;
const PREVIEW_HIGHLIGHT_BG: u32 = 0xfde04766;
const PREVIEW_SUPER_SUB_COLOR: u32 = 0x64748b;

/// Builds selection highlight quads for a byte range inside a shaped
/// [`TextLayout`], mirroring the source editor's wrap-aware selection paint.
pub(super) fn preview_selection_paint_quads(
    layout: &TextLayout,
    range: Range<usize>,
) -> Vec<PaintQuad> {
    if range.is_empty() {
        return Vec::new();
    }
    let bounds = layout.bounds();
    let line_height = layout.line_height();
    let text_len = layout.len();
    let start = range.start.min(text_len);
    let end = range.end.min(text_len);
    if start >= end {
        return Vec::new();
    }

    let Some(start_pos) = layout.position_for_index(start) else {
        return Vec::new();
    };
    let end_pos = layout
        .position_for_index(end)
        .unwrap_or_else(|| point(bounds.right(), start_pos.y));
    let selection_color = rgba(PREVIEW_SELECTION_COLOR);
    let mut quads = Vec::new();
    if start_pos.y == end_pos.y {
        quads.push(fill(
            Bounds::from_corners(
                point(start_pos.x, start_pos.y),
                point(end_pos.x.max(start_pos.x), start_pos.y + line_height),
            ),
            selection_color,
        ));
    } else {
        quads.push(fill(
            Bounds::from_corners(
                point(start_pos.x, start_pos.y),
                point(bounds.right(), start_pos.y + line_height),
            ),
            selection_color,
        ));
        let mid_top = start_pos.y + line_height;
        if end_pos.y > mid_top {
            quads.push(fill(
                Bounds::from_corners(
                    point(bounds.left(), mid_top),
                    point(bounds.right(), end_pos.y),
                ),
                selection_color,
            ));
        }
        quads.push(fill(
            Bounds::from_corners(
                point(bounds.left(), end_pos.y),
                point(end_pos.x, end_pos.y + line_height),
            ),
            selection_color,
        ));
    }
    quads
}

/// Index into shaped text for a pointer position. Falls back to the nearest
/// boundary when the pointer is outside the glyph bounds (above/below/side).
pub(super) fn preview_index_for_position(layout: &TextLayout, position: Point<Pixels>) -> usize {
    match layout.index_for_position(position) {
        Ok(index) => index,
        Err(index) => index,
    }
}

#[derive(Clone)]
pub(super) struct VisualTextSegment {
    pub(super) visible_range: Range<usize>,
    pub(super) source_range: Range<usize>,
}

/// Shaped text whose visible byte positions map back to canonical Markdown
/// byte positions. A click updates the existing source selection, so all
/// keyboard, clipboard, IME, undo, and formatting actions keep using the
/// source editor's mutation path.
struct VisualEditableText {
    element_id: ElementId,
    text: StyledText,
    segments: Vec<VisualTextSegment>,
    source_selection: Range<usize>,
    entity: Entity<MarkionApp>,
}

pub(super) fn visual_source_for_visible(segments: &[VisualTextSegment], visible: usize) -> usize {
    let Some(first) = segments.first() else {
        return 0;
    };
    for segment in segments {
        if visible <= segment.visible_range.end {
            let local = visible.saturating_sub(segment.visible_range.start);
            return segment.source_range.start + local.min(segment.source_range.len());
        }
    }
    segments
        .last()
        .map_or(first.source_range.start, |segment| segment.source_range.end)
}

pub(super) fn visual_visible_for_source(
    segments: &[VisualTextSegment],
    source: usize,
) -> Option<usize> {
    for segment in segments {
        if source >= segment.source_range.start && source <= segment.source_range.end {
            return Some(
                segment.visible_range.start
                    + source
                        .saturating_sub(segment.source_range.start)
                        .min(segment.visible_range.len()),
            );
        }
    }
    None
}

impl Element for VisualEditableText {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        Some(self.element_id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.text.request_layout(None, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        self.text
            .prepaint(None, inspector_id, bounds, state, window, cx);
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        let layout = self.text.layout().clone();
        if self.source_selection.is_empty() {
            if let Some(index) =
                visual_visible_for_source(&self.segments, self.source_selection.start)
                && let Some(position) = layout.position_for_index(index)
            {
                window.paint_quad(fill(
                    Bounds::new(position, size(px(2.), px(22.))),
                    rgb(0x2563eb),
                ));
            }
        } else {
            for segment in &self.segments {
                let start = self.source_selection.start.max(segment.source_range.start);
                let end = self.source_selection.end.min(segment.source_range.end);
                if start < end {
                    let visible_start = segment.visible_range.start
                        + start.saturating_sub(segment.source_range.start);
                    let visible_end = segment.visible_range.start
                        + end.saturating_sub(segment.source_range.start);
                    for quad in preview_selection_paint_quads(&layout, visible_start..visible_end) {
                        window.paint_quad(quad);
                    }
                }
            }
        }

        let entity = self.entity.clone();
        let segments = self.segments.clone();
        let text_layout = layout.clone();
        let hitbox_for_down = hitbox.clone();
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble
                || event.button != MouseButton::Left
                || !hitbox_for_down.is_hovered(window)
            {
                return;
            }
            let visible = preview_index_for_position(&text_layout, event.position);
            let source = visual_source_for_visible(&segments, visible);
            let focus_handle = entity.read(cx).focus_handle.clone();
            window.focus(&focus_handle);
            let _ = entity.update(cx, |app, cx| {
                app.file_tree_query_focused = false;
                app.search_focus = None;
                app.input_marked_len = 0;
                app.active_tab_mut().clear_preview_selection();
                app.active_tab_mut().is_selecting = true;
                if event.modifiers.shift {
                    app.select_to(source, cx);
                } else {
                    app.move_to(source, cx);
                }
            });
            window.refresh();
        });

        let entity = self.entity.clone();
        let segments = self.segments.clone();
        let text_layout = layout.clone();
        let hitbox_for_move = hitbox.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble
                || !event.dragging()
                || !hitbox_for_move.is_hovered(window)
                || !entity.read(cx).active_tab().is_selecting
            {
                return;
            }
            let visible = preview_index_for_position(&text_layout, event.position);
            let source = visual_source_for_visible(&segments, visible);
            let _ = entity.update(cx, |app, cx| app.select_to(source, cx));
        });

        let entity = self.entity.clone();
        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Bubble {
                let _ = entity.update(cx, |app, _| {
                    app.active_tab_mut().is_selecting = false;
                });
            }
        });

        window.set_cursor_style(CursorStyle::IBeam, hitbox);
        self.text
            .paint(None, inspector_id, bounds, &mut (), &mut (), window, cx);
    }
}

impl IntoElement for VisualEditableText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Selectable preview text: paints [`StyledText`], supports drag-selection into
/// app state, optional link clicks (only when the gesture did not create a
/// meaningful selection), and selection highlight quads.
struct SelectablePreviewText {
    element_id: ElementId,
    text: StyledText,
    block_index: usize,
    run_id: PreviewTextRunId,
    run_text: SharedString,
    selection_range: Option<Range<usize>>,
    link_ranges: Vec<Range<usize>>,
    link_urls: Vec<String>,
    entity: Entity<MarkionApp>,
}

impl SelectablePreviewText {
    fn new(
        id: impl Into<ElementId>,
        text: StyledText,
        block_index: usize,
        run_id: PreviewTextRunId,
        run_text: impl Into<SharedString>,
        selection_range: Option<Range<usize>>,
        entity: Entity<MarkionApp>,
    ) -> Self {
        Self {
            element_id: id.into(),
            text,
            block_index,
            run_id,
            run_text: run_text.into(),
            selection_range,
            link_ranges: Vec::new(),
            link_urls: Vec::new(),
            entity,
        }
    }

    fn with_links(mut self, ranges: Vec<Range<usize>>, urls: Vec<String>) -> Self {
        self.link_ranges = ranges;
        self.link_urls = urls;
        self
    }
}

impl Element for SelectablePreviewText {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        Some(self.element_id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.text.request_layout(None, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        self.text
            .prepaint(None, inspector_id, bounds, state, window, cx);
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        let text_layout = self.text.layout().clone();
        if let Some(range) = self.selection_range.clone() {
            for quad in preview_selection_paint_quads(&text_layout, range) {
                window.paint_quad(quad);
            }
        }

        let entity = self.entity.clone();
        let block_index = self.block_index;
        let run_id = self.run_id;
        let run_text = self.run_text.clone();
        let link_ranges = self.link_ranges.clone();
        let link_urls = self.link_urls.clone();

        // While a drag is active, every run arms mouse-up so the gesture can
        // finish even if the pointer left the anchor run. Otherwise arm down.
        let is_selecting = entity.read(cx).active_tab().preview_is_selecting;
        let drag_anchor_offset = entity
            .read(cx)
            .active_tab()
            .preview_selection
            .as_ref()
            .map(|sel| sel.anchor.offset);

        if is_selecting {
            let hitbox = hitbox.clone();
            let text_layout = text_layout.clone();
            let entity = entity.clone();
            let run_text = run_text.clone();
            let link_ranges = link_ranges.clone();
            let link_urls = link_urls.clone();
            window.on_mouse_event(
                move |event: &MouseUpEvent, phase, window: &mut Window, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    let up_index = preview_index_for_position(&text_layout, event.position);
                    let _ = entity.update(cx, |app, cx| {
                        if app.active_tab().preview_is_selecting && hitbox.is_hovered(window) {
                            app.update_preview_selection_head(
                                block_index,
                                run_id,
                                up_index,
                                run_text.clone(),
                                cx,
                            );
                        }
                        app.end_preview_selection(cx);

                        let blocks = app.active_tab().preview_list_blocks.clone();
                        let selection_empty = app
                            .active_tab()
                            .preview_selection
                            .as_ref()
                            .and_then(|sel| preview_selection_plain_text(sel, &blocks))
                            .is_none();
                        if selection_empty && hitbox.is_hovered(window) {
                            if let Some(anchor) = drag_anchor_offset {
                                for (range, url) in link_ranges.iter().zip(link_urls.iter()) {
                                    if range.contains(&anchor) && range.contains(&up_index) {
                                        cx.open_url(url);
                                        break;
                                    }
                                }
                            }
                        }
                    });
                    window.refresh();
                },
            );
        } else {
            let hitbox = hitbox.clone();
            let text_layout = text_layout.clone();
            let entity = entity.clone();
            let run_text = run_text.clone();
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || event.button != MouseButton::Left
                    || !hitbox.is_hovered(window)
                {
                    return;
                }
                let index = preview_index_for_position(&text_layout, event.position);
                let _ = entity.update(cx, |app, cx| {
                    app.begin_preview_selection(block_index, run_id, index, run_text.clone(), cx);
                });
                window.refresh();
            });
        }

        // Any run under the pointer may update head during a drag (cross-block).
        window.on_mouse_event({
            let hitbox = hitbox.clone();
            let text_layout = text_layout.clone();
            let entity = entity.clone();
            let run_text = run_text.clone();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !event.dragging() {
                    return;
                }
                if !entity.read(cx).active_tab().preview_is_selecting {
                    return;
                }
                if !hitbox.is_hovered(window) {
                    return;
                }
                let index = preview_index_for_position(&text_layout, event.position);
                let _ = entity.update(cx, |app, cx| {
                    app.update_preview_selection_head(
                        block_index,
                        run_id,
                        index,
                        run_text.clone(),
                        cx,
                    );
                });
            }
        });

        if !link_ranges.is_empty() {
            let mouse_position = window.mouse_position();
            if let Ok(ix) = text_layout.index_for_position(mouse_position)
                && link_ranges.iter().any(|range| range.contains(&ix))
            {
                window.set_cursor_style(CursorStyle::PointingHand, hitbox);
            }
        }

        // Right-click opens the preview context menu; resolve link under cursor.
        window.on_mouse_event({
            let hitbox = hitbox.clone();
            let text_layout = text_layout.clone();
            let entity = entity.clone();
            let link_ranges = link_ranges.clone();
            let link_urls = link_urls.clone();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || event.button != MouseButton::Right
                    || !hitbox.is_hovered(window)
                {
                    return;
                }
                let index = preview_index_for_position(&text_layout, event.position);
                let mut link_url = None;
                for (range, url) in link_ranges.iter().zip(link_urls.iter()) {
                    if range.contains(&index) {
                        link_url = Some(url.clone());
                        break;
                    }
                }
                let _ = entity.update(cx, |app, cx| {
                    app.show_preview_context_menu(event.position, link_url, cx);
                });
                window.refresh();
            }
        });

        self.text
            .paint(None, inspector_id, bounds, &mut (), &mut (), window, cx);
    }
}

impl IntoElement for SelectablePreviewText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Highlight byte range for a preview text run under the active free-range
/// selection, if that run intersects the selection.
pub(super) fn active_preview_run_selection(
    app: &MarkionApp,
    block_index: usize,
    run_id: PreviewTextRunId,
    run_text: &str,
) -> Option<Range<usize>> {
    app.active_tab()
        .preview_selection
        .as_ref()
        .and_then(|sel| preview_run_highlight_range(sel, block_index, run_id, run_text))
}

/// Renders block-level rich text as one selectable shaped text element, mapping
/// the document's inline spans (bold, italic, code, links, ...) to text runs.
/// Link spans open in the system browser when the click does not create a
/// meaningful text selection.
pub(super) fn rich_text_element(
    app: &MarkionApp,
    id: ElementId,
    rich: &RichText,
    block_index: usize,
    run_id: PreviewTextRunId,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let mut highlights: Vec<(Range<usize>, HighlightStyle)> = Vec::new();
    let mut link_ranges: Vec<Range<usize>> = Vec::new();
    let mut link_urls: Vec<String> = Vec::new();
    let mut offset = 0usize;

    for span in &rich.spans {
        let range = offset..offset + span.text.len();
        offset = range.end;

        let mut style = HighlightStyle::default();
        let mut styled = false;
        if span.style.bold {
            style.font_weight = Some(FontWeight::BOLD);
            styled = true;
        }
        if span.style.italic {
            style.font_style = Some(FontStyle::Italic);
            styled = true;
        }
        if span.style.strikethrough {
            style.strikethrough = Some(StrikethroughStyle {
                thickness: px(1.),
                color: None,
            });
            styled = true;
        }
        if span.style.code {
            style.background_color = Some(rgba(PREVIEW_INLINE_CODE_BG).into());
            style.color = Some(rgb(PREVIEW_INLINE_CODE_COLOR).into());
            styled = true;
        }
        if span.style.highlight {
            style.background_color = Some(rgba(PREVIEW_HIGHLIGHT_BG).into());
            styled = true;
        }
        if span.style.superscript || span.style.subscript {
            style.color = Some(rgb(PREVIEW_SUPER_SUB_COLOR).into());
            styled = true;
        }
        if let Some(url) = &span.link {
            style.color = Some(rgb(PREVIEW_LINK_COLOR).into());
            style.underline = Some(UnderlineStyle {
                thickness: px(1.),
                color: None,
                wavy: false,
            });
            styled = true;
            link_ranges.push(range.clone());
            link_urls.push(url.clone());
        }
        if styled {
            highlights.push((range, style));
        }
    }

    let styled_text =
        StyledText::new(SharedString::from(rich.text.clone())).with_highlights(highlights);
    let selection = active_preview_run_selection(app, block_index, run_id, &rich.text);
    SelectablePreviewText::new(
        id,
        styled_text,
        block_index,
        run_id,
        rich.text.clone(),
        selection,
        cx.entity().clone(),
    )
    .with_links(link_ranges, link_urls)
    .into_any_element()
}

/// Selectable plain / highlighted preview text (code, captions, table cells).
pub(super) fn selectable_plain_text(
    app: &MarkionApp,
    id: ElementId,
    styled: StyledText,
    plain: impl Into<SharedString>,
    block_index: usize,
    run_id: PreviewTextRunId,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let plain = plain.into();
    let selection = active_preview_run_selection(app, block_index, run_id, plain.as_ref());
    SelectablePreviewText::new(
        id,
        styled,
        block_index,
        run_id,
        plain,
        selection,
        cx.entity().clone(),
    )
    .into_any_element()
}

/// One shaped line of highlighted code (used when line numbers are shown).
pub(super) fn code_line_text(line: &[HighlightedSpan]) -> (StyledText, String) {
    let mut text = String::new();
    let mut highlights = Vec::new();
    for span in line {
        let start = text.len();
        text.push_str(&span.text);
        if span.kind != HighlightKind::Plain {
            highlights.push((
                start..text.len(),
                HighlightStyle {
                    color: Some(highlight_color(span.kind).into()),
                    ..HighlightStyle::default()
                },
            ));
        }
    }
    let plain = text.clone();
    if text.is_empty() {
        text.push(' ');
    }
    (
        StyledText::new(SharedString::from(text)).with_highlights(highlights),
        plain,
    )
}

/// All highlighted code lines joined into a single shaped text element (used
/// when line numbers are hidden); one element instead of one per token.
pub(super) fn code_block_text(lines: &[Vec<HighlightedSpan>]) -> (StyledText, String) {
    let mut text = String::new();
    let mut highlights = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            text.push('\n');
        }
        for span in line {
            let start = text.len();
            text.push_str(&span.text);
            if span.kind != HighlightKind::Plain {
                highlights.push((
                    start..text.len(),
                    HighlightStyle {
                        color: Some(highlight_color(span.kind).into()),
                        ..HighlightStyle::default()
                    },
                ));
            }
        }
    }
    let plain = text.clone();
    if text.is_empty() {
        text.push(' ');
    }
    (
        StyledText::new(SharedString::from(text)).with_highlights(highlights),
        plain,
    )
}

/// Compute the minimal [`ListState::splice`] arguments to turn `old` into
/// `new`: the range of `old` indices that changed, and how many `new` items
/// replace them. Found via a common-prefix / common-suffix scan, which is exact
/// for the localized edits typing produces (one or a few adjacent blocks change)
/// and always correct — an identical slice yields an empty range and zero count.
pub(super) fn preview_block_splice(
    old: &[PreviewBlock],
    new: &[PreviewBlock],
) -> (std::ops::Range<usize>, usize) {
    block_splice(old, new)
}

pub(super) fn block_splice<T: PartialEq>(old: &[T], new: &[T]) -> (std::ops::Range<usize>, usize) {
    let max_prefix = old.len().min(new.len());
    let mut prefix = 0;
    while prefix < max_prefix && old[prefix] == new[prefix] {
        prefix += 1;
    }
    // Longest common suffix, bounded so it cannot overlap the shared prefix in
    // the shorter slice.
    let max_suffix = max_prefix - prefix;
    let mut suffix = 0;
    while suffix < max_suffix && old[old.len() - 1 - suffix] == new[new.len() - 1 - suffix] {
        suffix += 1;
    }
    (prefix..(old.len() - suffix), new.len() - suffix - prefix)
}

/// Decide whether a render with a stale preview should parse now or keep
/// showing the previous blocks. Callers only ask when the preview IS stale
/// (blocks don't reflect the current document version).
///
/// Parse when typing has settled (`since_change` has outlived the debounce) or
/// when the last parse is so old that waiting longer would visibly freeze the
/// preview (`since_parse` past `PREVIEW_MAX_STALE`). `None` means "never":
/// never-changed (first render of a document) and never-parsed both must parse
/// immediately.
pub(super) fn should_parse_preview_now(
    since_change: Option<Duration>,
    since_parse: Option<Duration>,
) -> bool {
    let settled = since_change.is_none_or(|d| d >= PREVIEW_DEBOUNCE);
    let too_stale = since_parse.is_none_or(|d| d >= PREVIEW_MAX_STALE);
    settled || too_stale
}

/// Globally unique id for a background preview parse (see
/// `EditorTab::preview_parse_inflight`). Global uniqueness is what lets a
/// landing result safely locate its owning tab: `text_version`s can collide
/// across documents, but two tabs can never carry the same task id.
pub(super) fn next_preview_parse_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

pub(super) fn visual_highlight_style(run: &VisualInlineRun) -> Option<HighlightStyle> {
    let mut style = HighlightStyle::default();
    let mut styled = false;
    if run.style.bold {
        style.font_weight = Some(FontWeight::BOLD);
        styled = true;
    }
    if run.style.italic {
        style.font_style = Some(FontStyle::Italic);
        styled = true;
    }
    if run.style.strikethrough {
        style.strikethrough = Some(StrikethroughStyle {
            thickness: px(1.),
            color: None,
        });
        styled = true;
    }
    if run.style.code {
        style.background_color = Some(rgba(PREVIEW_INLINE_CODE_BG).into());
        style.color = Some(rgb(PREVIEW_INLINE_CODE_COLOR).into());
        styled = true;
    }
    if run.link_target_range.is_some() {
        style.color = Some(rgb(PREVIEW_LINK_COLOR).into());
        style.underline = Some(UnderlineStyle {
            thickness: px(1.),
            color: None,
            wavy: false,
        });
        styled = true;
    }
    styled.then_some(style)
}

pub(super) fn visual_text_element(
    block: &VisualBlock,
    block_index: usize,
    app: &MarkionApp,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let mut visible = String::new();
    let mut highlights = Vec::new();
    let mut segments = Vec::new();
    for run in &block.editable_runs {
        let start = visible.len();
        visible.push_str(&run.visible_text);
        let range = start..visible.len();
        if let Some(style) = visual_highlight_style(run) {
            highlights.push((range.clone(), style));
        }
        segments.push(VisualTextSegment {
            visible_range: range,
            source_range: run.content_range.clone(),
        });
    }
    VisualEditableText {
        element_id: ElementId::from(("visual-text", block_index)),
        text: StyledText::new(SharedString::from(visible)).with_highlights(highlights),
        segments,
        source_selection: app.active_tab().selected_range.clone(),
        entity: cx.entity(),
    }
    .into_any_element()
}

pub(super) fn visual_source_island_view(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let source = app.active_tab().document.text()[block.source_range.clone()].to_string();
    let source_len = source.len();
    div()
        .mb_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(rgb(0xcbd5e1))
        .bg(rgb(0xf8fafc))
        .font_family("JetBrains Mono")
        .text_size(px(13.))
        .line_height(px(21.))
        .child(VisualEditableText {
            element_id: ElementId::from(("visual-source-island", block_index)),
            text: StyledText::new(SharedString::from(source)),
            segments: vec![VisualTextSegment {
                visible_range: 0..source_len,
                source_range: block.source_range.clone(),
            }],
            source_selection: app.active_tab().selected_range.clone(),
            entity: cx.entity(),
        })
}

pub(super) fn visual_block_is_focused(app: &MarkionApp, block: &VisualBlock) -> bool {
    let cursor = app.active_tab().cursor_offset();
    visual_source_range_is_focused(
        &block.source_range,
        cursor,
        app.active_tab().document.text().len(),
    )
}

pub(super) fn visual_source_range_is_focused(
    source_range: &Range<usize>,
    cursor: usize,
    document_len: usize,
) -> bool {
    source_range.contains(&cursor) || (cursor == document_len && cursor == source_range.end)
}

pub(super) fn visual_block_view(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    document_dir: Option<&Path>,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let focused = visual_block_is_focused(app, block);
    let always_source = matches!(
        block.source_island,
        Some(
            VisualSourceIslandKind::FrontMatter
                | VisualSourceIslandKind::Code
                | VisualSourceIslandKind::Math
                | VisualSourceIslandKind::Html
                | VisualSourceIslandKind::Unsupported
        )
    ) || block
        .editable_runs
        .iter()
        .any(|run| run.conservative_fallback);
    if focused || always_source {
        return visual_source_island_view(app, block, block_index, cx);
    }

    match &block.kind {
        VisualBlockKind::Heading { level } => {
            let size = match level {
                1 => px(24.),
                2 => px(20.),
                3 => px(18.),
                _ => px(16.),
            };
            div()
                .mt_2()
                .mb_2()
                .text_size(size)
                .font_weight(FontWeight::BOLD)
                .child(visual_text_element(block, block_index, app, cx))
        }
        VisualBlockKind::Paragraph => div()
            .mb_3()
            .line_height(px(24.))
            .text_size(px(14.))
            .child(visual_text_element(block, block_index, app, cx)),
        VisualBlockKind::ListItem {
            level,
            ordered,
            index,
            checked,
        } => {
            let marker = match checked {
                Some(true) => "☑".to_string(),
                Some(false) => "☐".to_string(),
                None if *ordered => format!("{}.", index.unwrap_or(1)),
                None => match level {
                    1 => "•".to_string(),
                    2 => "◦".to_string(),
                    _ => "▪".to_string(),
                },
            };
            div()
                .mb_1()
                .ml(px((*level as f32 - 1.).max(0.) * 18.))
                .text_size(px(14.))
                .line_height(px(22.))
                .flex()
                .items_start()
                .child(
                    div()
                        .flex_none()
                        .min_w(px(22.))
                        .pr_1()
                        .text_color(rgb(0x64748b))
                        .child(marker),
                )
                .child(div().flex_1().min_w_0().child(visual_text_element(
                    block,
                    block_index,
                    app,
                    cx,
                )))
        }
        VisualBlockKind::BlockQuote => div()
            .mb_3()
            .pl_3()
            .border_l_1()
            .border_color(rgb(0x94a3b8))
            .text_color(rgb(0x475569))
            .line_height(px(23.))
            .child(visual_text_element(block, block_index, app, cx)),
        VisualBlockKind::Image { alt, url, title } => {
            let offset = block.source_range.start;
            let caption = if alt.is_empty() {
                url.as_str()
            } else {
                alt.as_str()
            };
            div()
                .mb_3()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(rgb(0xcbd5e1))
                .bg(rgb(0xf8fafc))
                .cursor(CursorStyle::PointingHand)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |app, _, _, cx| app.move_to(offset, cx)),
                )
                .child(
                    div()
                        .rounded_md()
                        .overflow_hidden()
                        .bg(rgb(0xffffff))
                        .child(img(preview_image_source(url, document_dir)).max_w_full()),
                )
                .child(
                    div()
                        .mt_2()
                        .text_size(px(12.))
                        .text_color(rgb(0x475569))
                        .child(format!("Image: {caption}")),
                )
                .children(title.as_ref().map(|title| {
                    div()
                        .mt_1()
                        .text_size(px(11.))
                        .text_color(rgb(0x64748b))
                        .child(title.clone())
                }))
        }
        VisualBlockKind::Rule => {
            let offset = block.source_range.start;
            div()
                .my_3()
                .h(px(12.))
                .cursor(CursorStyle::IBeam)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |app, _, _, cx| app.move_to(offset, cx)),
                )
                .child(div().mt(px(5.)).h(px(1.)).bg(rgb(0xcbd5e1)))
        }
        VisualBlockKind::Table { rows, .. } => {
            visual_table_view(app, rows, block.source_range.start, cx)
        }
        VisualBlockKind::CodeBlock { .. }
        | VisualBlockKind::MathBlock
        | VisualBlockKind::Unsupported => visual_source_island_view(app, block, block_index, cx),
    }
}

type TableToolbarAction = (&'static str, TableEdit, Msg);

const VISUAL_TABLE_TOOLBAR_ACTIONS: [TableToolbarAction; 6] = [
    ("+Row", TableEdit::AddRow, Msg::StatusFmtAddRow),
    ("-Row", TableEdit::DeleteRow, Msg::StatusFmtDeleteRow),
    ("Up", TableEdit::MoveRowUp, Msg::StatusFmtMoveRowUp),
    ("Down", TableEdit::MoveRowDown, Msg::StatusFmtMoveRowDown),
    ("+Col", TableEdit::AddColumn, Msg::StatusFmtAddColumn),
    ("-Col", TableEdit::DeleteColumn, Msg::StatusFmtDeleteColumn),
];

pub(super) fn table_toolbar_actions_for_view_mode(
    view_mode: ViewMode,
) -> &'static [TableToolbarAction] {
    if matches!(view_mode, ViewMode::VisualEdit) {
        &VISUAL_TABLE_TOOLBAR_ACTIONS
    } else {
        &[]
    }
}

pub(super) fn visual_table_view(
    app: &MarkionApp,
    rows: &[Vec<String>],
    table_offset: usize,
    cx: &mut Context<MarkionApp>,
) -> Div {
    div()
        .mb_3()
        .border_1()
        .border_color(rgb(0xcbd5e1))
        .rounded_md()
        .overflow_hidden()
        .child(
            div()
                .px_2()
                .py_1()
                .flex()
                .gap_1()
                .items_center()
                .bg(rgb(0xf8fafc))
                .border_b_1()
                .border_color(rgb(0xe2e8f0))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(11.))
                        .text_color(rgb(0x64748b))
                        .child(app.tr(Msg::LabelTable)),
                )
                .children(
                    table_toolbar_actions_for_view_mode(ViewMode::VisualEdit)
                        .iter()
                        .map(|&(label, edit, status)| {
                            preview_table_button(label, table_offset, edit, status, cx)
                        }),
                ),
        )
        .children(rows.iter().enumerate().map(|(row_index, row)| {
            let background = if row_index == 0 {
                rgb(0xf1f5f9)
            } else {
                rgb(0xffffff)
            };
            let is_last_row = row_index + 1 == rows.len();
            div()
                .flex()
                .bg(background)
                .when(!is_last_row, |style| {
                    style.border_b_1().border_color(rgb(0xe2e8f0))
                })
                .children(row.iter().enumerate().map(|(cell_index, cell)| {
                    let is_last_cell = cell_index + 1 == row.len();
                    let offset = table_offset;
                    div()
                        .flex_1()
                        .min_w_0()
                        .p_2()
                        .when(!is_last_cell, |style| {
                            style.border_r_1().border_color(rgb(0xe2e8f0))
                        })
                        .text_size(px(12.))
                        .cursor(CursorStyle::IBeam)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |app, _, _, cx| app.move_to(offset, cx)),
                        )
                        .child(cell.clone())
                }))
        }))
}

fn html_preview_block_view(
    app: &MarkionApp,
    html: &str,
    block_index: usize,
    document_dir: Option<&Path>,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let parts = html_preview_parts(html);
    if parts.is_empty() {
        return div();
    }

    div()
        .mb_3()
        .children(parts.into_iter().enumerate().map(|(part_index, part)| {
            match part {
                HtmlPreviewPart::Text { text, centered } => div()
                    .mb_2()
                    .line_height(px(24.))
                    .text_size(px(14.))
                    .when(centered, |style| style.text_center())
                    .child(rich_text_element(
                        app,
                        ElementId::from((
                            "preview-html-text",
                            ((block_index as u64) << 32) | part_index as u64,
                        )),
                        &text,
                        block_index,
                        PreviewTextRunId::HtmlText,
                        cx,
                    )),
                HtmlPreviewPart::Image { url, centered, .. } => div()
                    .mb_2()
                    .when(centered, |style| style.flex().justify_center())
                    .child(img(preview_image_source(&url, document_dir)).max_w_full()),
            }
        }))
}

pub(super) fn preview_block_view(
    app: &MarkionApp,
    block: &PreviewBlock,
    block_index: usize,
    document_dir: Option<&Path>,
    show_code_line_numbers: bool,
    cx: &mut Context<MarkionApp>,
) -> Div {
    match block {
        PreviewBlock::Heading { level, text, .. } => {
            let size = match level {
                1 => px(24.),
                2 => px(20.),
                3 => px(18.),
                _ => px(16.),
            };
            div()
                .mt_2()
                .mb_2()
                .text_size(size)
                .font_weight(gpui::FontWeight::BOLD)
                .child(rich_text_element(
                    app,
                    ElementId::from(("preview-heading", block_index)),
                    text,
                    block_index,
                    PreviewTextRunId::Body,
                    cx,
                ))
        }
        PreviewBlock::Paragraph { text, .. } => div()
            .mb_3()
            .line_height(px(24.))
            .text_size(px(14.))
            .child(rich_text_element(
                app,
                ElementId::from(("preview-paragraph", block_index)),
                text,
                block_index,
                PreviewTextRunId::Body,
                cx,
            )),
        PreviewBlock::ListItem {
            level,
            ordered,
            index,
            checked,
            text,
            ..
        } => {
            let marker = match checked {
                Some(true) => "☑".to_string(),
                Some(false) => "☐".to_string(),
                None if *ordered => format!("{}.", index.unwrap_or(1)),
                None => match level {
                    1 => "•".to_string(),
                    2 => "◦".to_string(),
                    _ => "▪".to_string(),
                },
            };
            let marker_color = match checked {
                Some(true) => rgb(0x16a34a),
                Some(false) => rgb(0x64748b),
                None => rgb(0x64748b),
            };
            div()
                .mb_1()
                .ml(px((*level as f32 - 1.).max(0.) * 18.))
                .text_size(px(14.))
                .line_height(px(22.))
                .flex()
                .items_start()
                .child(
                    div()
                        .flex_none()
                        .min_w(px(22.))
                        .pr_1()
                        .text_color(marker_color)
                        .child(marker),
                )
                .child(div().flex_1().min_w_0().child(rich_text_element(
                    app,
                    ElementId::from(("preview-list-item", block_index)),
                    text,
                    block_index,
                    PreviewTextRunId::Body,
                    cx,
                )))
        }
        PreviewBlock::BlockQuote { text, .. } => div()
            .mb_3()
            .pl_3()
            .border_l_1()
            .border_color(rgb(0x94a3b8))
            .text_color(rgb(0x475569))
            .line_height(px(23.))
            .child(rich_text_element(
                app,
                ElementId::from(("preview-quote", block_index)),
                text,
                block_index,
                PreviewTextRunId::Body,
                cx,
            )),
        PreviewBlock::CodeBlock { language, code, .. } => {
            let highlighted = app.highlighted_code(language.as_deref(), code);
            let body = div()
                .mb_3()
                .p_3()
                .rounded_md()
                .bg(rgb(0x0f172a))
                .text_color(rgb(0xe2e8f0))
                .font_family("JetBrains Mono")
                .text_size(px(12.))
                .line_height(px(19.))
                .children(language.as_ref().map(|language| {
                    div()
                        .mb_2()
                        .text_size(px(11.))
                        .text_color(rgb(0x93c5fd))
                        .child(language.clone())
                }));
            if show_code_line_numbers {
                body.children(highlighted.iter().enumerate().map(|(line_index, line)| {
                    let (styled, plain) = code_line_text(line);
                    div()
                        .flex()
                        .items_start()
                        .child(
                            div()
                                .w(px(36.))
                                .flex_none()
                                .pr_2()
                                .text_color(rgb(0x64748b))
                                .child(format!("{:>3}", line_index + 1)),
                        )
                        .child(div().flex_1().min_w_0().child(selectable_plain_text(
                            app,
                            ElementId::from((
                                "preview-code-line",
                                ((block_index as u64) << 32) | (line_index as u64),
                            )),
                            styled,
                            plain,
                            block_index,
                            PreviewTextRunId::CodeLine(line_index),
                            cx,
                        )))
                }))
            } else {
                let (styled, plain) = code_block_text(&highlighted);
                body.child(selectable_plain_text(
                    app,
                    ElementId::from(("preview-code", block_index)),
                    styled,
                    plain,
                    block_index,
                    PreviewTextRunId::CodeBody,
                    cx,
                ))
            }
        }
        PreviewBlock::MathBlock { latex, error, .. } => {
            let rendered = render_math(latex, true);
            let rendered_plain = rendered.text.clone();
            let panel = div()
                .mb_3()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(if error.is_some() {
                    rgb(0xfca5a5)
                } else {
                    rgb(0xbfdbfe)
                })
                .bg(if error.is_some() {
                    rgb(0xfef2f2)
                } else {
                    rgb(0xeff6ff)
                })
                .font_family("Cambria Math")
                .text_size(px(16.))
                .line_height(px(24.))
                .child(selectable_plain_text(
                    app,
                    ElementId::from(("preview-math-rendered", block_index)),
                    StyledText::new(SharedString::from(rendered_plain.clone())),
                    rendered_plain,
                    block_index,
                    PreviewTextRunId::MathRendered,
                    cx,
                ))
                .child(
                    div()
                        .mt_2()
                        .text_size(px(11.))
                        .text_color(rgb(0x64748b))
                        .child(selectable_plain_text(
                            app,
                            ElementId::from(("preview-math-latex", block_index)),
                            StyledText::new(SharedString::from(latex.clone())),
                            latex.clone(),
                            block_index,
                            PreviewTextRunId::MathLatex,
                            cx,
                        )),
                );

            if let Some(error) = error {
                panel.child(
                    div()
                        .mt_2()
                        .text_size(px(12.))
                        .text_color(rgb(0xb91c1c))
                        .child(format!("Math error: {error}")),
                )
            } else {
                panel
            }
        }
        PreviewBlock::Html { html, .. } => {
            html_preview_block_view(app, html, block_index, document_dir, cx)
        }
        PreviewBlock::Image {
            alt, url, title, ..
        } => {
            let caption = if alt.is_empty() {
                url.as_str()
            } else {
                alt.as_str()
            };
            let caption_label = format!("Image: {caption}");
            let meta = if title.as_deref().unwrap_or("").is_empty() {
                url.clone()
            } else {
                format!("{url} - {}", title.as_deref().unwrap_or(""))
            };
            div()
                .mb_3()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(rgb(0xcbd5e1))
                .bg(rgb(0xf8fafc))
                .child(
                    div()
                        .rounded_md()
                        .overflow_hidden()
                        .bg(rgb(0xffffff))
                        .child(img(preview_image_source(url, document_dir)).max_w_full()),
                )
                .child(
                    div()
                        .mt_2()
                        .text_size(px(12.))
                        .text_color(rgb(0x475569))
                        .child(selectable_plain_text(
                            app,
                            ElementId::from(("preview-image-caption", block_index)),
                            StyledText::new(SharedString::from(caption_label.clone())),
                            caption_label,
                            block_index,
                            PreviewTextRunId::ImageCaption,
                            cx,
                        )),
                )
                .child(
                    div()
                        .mt_1()
                        .text_size(px(11.))
                        .text_color(rgb(0x64748b))
                        .child(selectable_plain_text(
                            app,
                            ElementId::from(("preview-image-meta", block_index)),
                            StyledText::new(SharedString::from(meta.clone())),
                            meta,
                            block_index,
                            PreviewTextRunId::ImageMeta,
                            cx,
                        )),
                )
        }
        PreviewBlock::Rule { .. } => div().my_3().h(px(1.)).bg(rgb(0xcbd5e1)),
        PreviewBlock::Table { rows, .. } => {
            // Split Preview and Read mode share this branch. Table mutation
            // belongs in Visual Edit or the source commands, so the preview
            // grid intentionally has no editing header or callbacks.
            div()
                .mb_3()
                .border_1()
                .border_color(rgb(0xcbd5e1))
                .rounded_md()
                .overflow_hidden()
                .children(rows.iter().enumerate().map(|(row_index, row)| {
                    let background = if row_index == 0 {
                        rgb(0xf1f5f9)
                    } else {
                        rgb(0xffffff)
                    };
                    let is_last_row = row_index + 1 == rows.len();
                    div()
                        .flex()
                        .bg(background)
                        .when(!is_last_row, |style| {
                            style.border_b_1().border_color(rgb(0xe2e8f0))
                        })
                        .children(row.iter().enumerate().map(|(cell_index, cell)| {
                            let is_last_cell = cell_index + 1 == row.len();
                            let cell_text = cell.clone();
                            div()
                                .flex_1()
                                .min_w_0()
                                .p_2()
                                .when(!is_last_cell, |style| {
                                    style.border_r_1().border_color(rgb(0xe2e8f0))
                                })
                                .text_size(px(12.))
                                .child(selectable_plain_text(
                                    app,
                                    ElementId::from((
                                        "preview-table-cell",
                                        ((block_index as u64) << 32)
                                            | (((row_index as u64) & 0xffff) << 16)
                                            | ((cell_index as u64) & 0xffff),
                                    )),
                                    StyledText::new(SharedString::from(cell_text.clone())),
                                    cell_text,
                                    block_index,
                                    PreviewTextRunId::TableCell {
                                        row: row_index,
                                        col: cell_index,
                                    },
                                    cx,
                                ))
                        }))
                }))
        }
    }
}

pub(super) fn preview_table_button(
    label: &'static str,
    table_offset: usize,
    edit: TableEdit,
    status: Msg,
    cx: &mut Context<MarkionApp>,
) -> Div {
    div()
        .flex_none()
        .px_2()
        .py_1()
        .rounded_sm()
        .border_1()
        .border_color(rgb(0xcbd5e1))
        .bg(rgb(0xffffff))
        .text_size(px(11.))
        .text_color(rgb(0x334155))
        .cursor_pointer()
        .child(label)
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                let tab = app.active_tab_mut();
                tab.selected_range = table_offset..table_offset;
                tab.selection_reversed = false;
                app.apply_table_edit_at(table_offset, edit, t(app.language, status).into(), cx);
            }),
        )
}

pub(super) fn preview_image_source(url: &str, document_dir: Option<&Path>) -> ImageSource {
    if is_remote_resource(url) {
        return url.to_string().into();
    }

    let path = PathBuf::from(url);
    let path = if path.is_absolute() {
        path
    } else if let Some(document_dir) = document_dir {
        document_dir.join(path)
    } else {
        path
    };
    path.into()
}

pub(super) fn is_remote_resource(url: &str) -> bool {
    url.contains("://") || url.starts_with("data:")
}

pub(super) fn highlight_color(kind: HighlightKind) -> Rgba {
    match kind {
        HighlightKind::Plain => rgb(0xe2e8f0),
        HighlightKind::Keyword => rgb(0xc084fc),
        HighlightKind::String => rgb(0x86efac),
        HighlightKind::Number => rgb(0xfbbf24),
        HighlightKind::Comment => rgb(0x94a3b8),
        HighlightKind::Type => rgb(0x67e8f9),
    }
}

pub(super) fn utf16_offset_to_byte_offset(text: &str, offset: usize) -> usize {
    let mut byte_offset = 0;
    let mut utf16_count = 0;

    for ch in text.chars() {
        if utf16_count >= offset {
            break;
        }
        utf16_count += ch.len_utf16();
        byte_offset += ch.len_utf8();
    }

    byte_offset
}

pub(super) fn byte_offset_to_utf16_offset(text: &str, offset: usize) -> usize {
    let offset = clamp_to_text_boundary(text, offset);
    let mut utf16_offset = 0;
    let mut byte_count = 0;

    for ch in text.chars() {
        if byte_count >= offset {
            break;
        }
        byte_count += ch.len_utf8();
        utf16_offset += ch.len_utf16();
    }

    utf16_offset
}

pub(super) fn clamp_to_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
