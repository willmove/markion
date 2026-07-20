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
        (PreviewBlock::MathBlock { authored, .. }, PreviewTextRunId::MathLatex) => {
            Some(authored.clone())
        }
        (PreviewBlock::Html { html, .. }, PreviewTextRunId::HtmlText) => {
            Some(html_preview_plain_text(html))
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
        PreviewBlock::MathBlock { .. } => vec![PreviewTextRunId::MathLatex],
        PreviewBlock::Html { html, .. } => (!html_preview_plain_text(html).is_empty())
            .then_some(PreviewTextRunId::HtmlText)
            .into_iter()
            .collect(),
        PreviewBlock::Image { .. } => Vec::new(),
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
    for (block_index, block) in blocks
        .iter()
        .enumerate()
        .take(end.block_index + 1)
        .skip(start.block_index)
    {
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
    for block in blocks
        .iter()
        .take(end.block_index + 1)
        .skip(start.block_index)
    {
        let range = preview_block_source_range(block)?;
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

/// Registers the existing source-backed input handler exactly once for the
/// Visual Edit surface. It deliberately creates no hitbox; visual rows keep
/// owning pointer-to-source mapping.
pub(super) struct VisualInputElement {
    pub(super) app: Entity<MarkionApp>,
}

impl IntoElement for VisualInputElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for VisualInputElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::from("visual-input-bridge"))
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.).into();
        style.size.height = gpui::relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _state: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.app.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.app.clone()),
            cx,
        );
        self.app.update(cx, |app, _| {
            app.active_tab_mut().visual_input_bounds = Some(bounds);
        });
    }
}

/// Shaped text whose visible byte positions map back to canonical Markdown
/// byte positions. A click updates the existing source selection, so all
/// keyboard, clipboard, IME, undo, and formatting actions keep using the
/// source editor's mutation path.
struct VisualEditableText {
    element_id: ElementId,
    block_index: usize,
    source_island: bool,
    text: StyledText,
    projection: VisualProjection,
    source_selection: Range<usize>,
    source_cursor: usize,
    marked_range: Option<Range<usize>>,
    /// Whether this row is the single owner of the document caret. Every
    /// visible row paints per frame, and the visible→source mapping clamps to
    /// its own segments, so an unfocused row would otherwise paint a stray
    /// caret at its nearest boundary.
    caret_active: bool,
    /// Focused multi-field blocks register geometry only for their active
    /// field, so sibling cells cannot overwrite the navigation snapshot.
    navigation_active: bool,
    /// Source position clicks resolve to when this row has no segments
    /// (an empty block still needs to place the caret inside itself).
    entity: Entity<MarkionApp>,
    #[cfg(test)]
    test_projection: Option<(String, Vec<Range<usize>>)>,
    #[cfg(test)]
    test_projection_styles: Option<Vec<InlineStyle>>,
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
        let affinity = self
            .entity
            .read(cx)
            .active_tab()
            .current_visual_caret_affinity();
        let caret_bounds = self
            .caret_active
            .then(|| {
                let display = self.projection.display_for_source(self.source_cursor)?;
                if let Some(affinity) = affinity {
                    let candidates = self.projection.boundary_candidates(display);
                    if candidates.is_ambiguous()
                        && candidates.resolve(affinity) != self.source_cursor
                    {
                        return self.projection.display_for_source(self.source_cursor);
                    }
                }
                Some(display)
            })
            .flatten()
            .and_then(|index| layout.position_for_index(index))
            .map(|position| Bounds::new(position, size(px(2.), layout.line_height())));
        if self.source_selection.is_empty() {
            if let Some(caret_bounds) = caret_bounds {
                #[cfg(test)]
                self.entity.update(cx, |app, _| {
                    app.active_tab_mut().visual_caret_paint_count += 1;
                });
                if self.entity.read(cx).focus_handle.is_focused(window) {
                    window.paint_quad(fill(caret_bounds, rgb(0x2563eb)));
                }
            }
        } else {
            for segment in &self.projection.segments {
                let start = self.source_selection.start.max(segment.source_range.start);
                let end = self.source_selection.end.min(segment.source_range.end);
                if start < end {
                    let visible_start = segment.display_range.start
                        + start
                            .saturating_sub(segment.source_range.start)
                            .min(segment.display_range.len());
                    let visible_end = segment.display_range.start
                        + end
                            .saturating_sub(segment.source_range.start)
                            .min(segment.display_range.len());
                    for quad in preview_selection_paint_quads(&layout, visible_start..visible_end) {
                        window.paint_quad(quad);
                    }
                }
            }
        }

        if let Some(caret_bounds) = caret_bounds {
            self.entity.update(cx, |app, _| {
                app.active_tab_mut().visual_caret_bounds = Some(caret_bounds);
            });
        }
        if let Some(marked_range) = self.marked_range.clone() {
            let mut marked_bounds: Option<Bounds<Pixels>> = None;
            for segment in &self.projection.segments {
                let start = marked_range.start.max(segment.source_range.start);
                let end = marked_range.end.min(segment.source_range.end);
                if start >= end {
                    continue;
                }
                let display_start = segment.display_range.start
                    + (start - segment.source_range.start).min(segment.display_range.len());
                let display_end = segment.display_range.start
                    + (end - segment.source_range.start).min(segment.display_range.len());
                for quad in preview_selection_paint_quads(&layout, display_start..display_end) {
                    marked_bounds = Some(
                        marked_bounds.map_or(quad.bounds, |bounds| bounds.union(&quad.bounds)),
                    );
                }
            }
            if let Some(marked_bounds) = marked_bounds {
                self.entity.update(cx, |app, _| {
                    app.active_tab_mut().visual_marked_range_bounds =
                        Some((marked_range, marked_bounds));
                });
            }
        }
        let document_version = self.entity.read(cx).active_tab().document.version();
        let marked_range = self.entity.read(cx).active_tab().marked_range.clone();
        if self.navigation_active {
            let navigation_snapshot = visual_navigation_snapshot(
                document_version,
                self.block_index,
                self.source_selection.clone(),
                marked_range,
                self.source_island,
                &self.projection,
                &layout,
            );
            self.entity.update(cx, |app, cx| {
                app.active_tab_mut()
                    .register_visual_navigation_snapshot(navigation_snapshot);
                app.complete_pending_visual_navigation(cx);
            });
        }
        #[cfg(test)]
        if let Some(projection) = self.test_projection.clone() {
            let styles = self.test_projection_styles.clone();
            self.entity.update(cx, move |app, _| {
                let tab = app.active_tab_mut();
                tab.visual_last_projection = Some(projection);
                tab.visual_last_projection_styles = styles;
                tab.visual_projection_paint_count += 1;
            });
        }

        let entity = self.entity.clone();
        let projection = self.projection.clone();
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
            let candidates = projection.boundary_candidates(visible);
            let boundary_x = text_layout
                .position_for_index(candidates.display_offset)
                .map(|position| position.x)
                .unwrap_or(event.position.x);
            let affinity = if candidates.is_ambiguous() && event.position.x < boundary_x {
                Some(VisualCaretAffinity::Upstream)
            } else if candidates.is_ambiguous() {
                Some(VisualCaretAffinity::Downstream)
            } else {
                None
            };
            let source = candidates.resolve(affinity.unwrap_or(VisualCaretAffinity::Downstream));
            let focus_handle = entity.read(cx).focus_handle.clone();
            window.focus(&focus_handle);
            entity.update(cx, |app, cx| {
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
                app.active_tab_mut().set_visual_caret_affinity(affinity);
            });
            window.refresh();
        });

        let entity = self.entity.clone();
        let projection = self.projection.clone();
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
            let candidates = projection.boundary_candidates(visible);
            let source = candidates.resolve(VisualCaretAffinity::Downstream);
            entity.update(cx, |app, cx| {
                app.select_to(source, cx);
                app.active_tab_mut().set_visual_caret_affinity(None);
            });
        });

        let entity = self.entity.clone();
        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Bubble {
                entity.update(cx, |app, _| {
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

fn visual_navigation_snapshot(
    document_version: u64,
    block_index: usize,
    source_selection: Range<usize>,
    marked_range: Option<Range<usize>>,
    source_island: bool,
    projection: &VisualProjection,
    layout: &TextLayout,
) -> VisualNavigationSnapshot {
    let bounds = layout.bounds();
    let line_height = layout.line_height();
    let mut line_ys = projection
        .text
        .grapheme_indices(true)
        .map(|(display, _)| display)
        .chain(std::iter::once(projection.text.len()))
        .filter_map(|display| layout.position_for_index(display).map(|point| point.y))
        .collect::<Vec<_>>();
    if line_ys.is_empty() {
        line_ys.push(bounds.top());
    }
    line_ys.sort_by(|left, right| left.to_f64().total_cmp(&right.to_f64()));
    line_ys.dedup();

    let display_boundaries = projection
        .text
        .grapheme_indices(true)
        .map(|(display, _)| display)
        .chain(std::iter::once(projection.text.len()))
        .collect::<Vec<_>>();
    let mut lines = Vec::with_capacity(line_ys.len());
    for y in line_ys {
        let sample_y = y + line_height * 0.5;
        let display_start = preview_index_for_position(layout, point(bounds.left(), sample_y));
        let display_end = preview_index_for_position(layout, point(bounds.right(), sample_y));
        let mut carets = Vec::new();
        for display in display_boundaries
            .iter()
            .copied()
            .filter(|display| *display >= display_start && *display <= display_end)
        {
            let position = layout.position_for_index(display);
            let x = if display == display_start {
                bounds.left()
            } else if let Some(position) = position.filter(|position| position.y == y) {
                position.x
            } else {
                continue;
            };
            let candidates = projection.boundary_candidates(display);
            carets.push(VisualNavigationCaret {
                source_offset: candidates.upstream_source,
                x,
            });
            if candidates.is_ambiguous() {
                carets.push(VisualNavigationCaret {
                    source_offset: candidates.downstream_source,
                    x,
                });
            }
        }
        if carets.is_empty() {
            carets.push(VisualNavigationCaret {
                source_offset: projection.source_anchor,
                x: bounds.left(),
            });
        }
        carets.sort_by(|left, right| {
            left.x
                .to_f64()
                .total_cmp(&right.x.to_f64())
                .then_with(|| left.source_offset.cmp(&right.source_offset))
        });
        carets.dedup();
        lines.push(VisualNavigationLine { y, carets });
    }

    VisualNavigationSnapshot {
        document_version,
        block_index,
        source_selection,
        marked_range,
        source_island,
        lines,
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
    /// Byte offset of this shaped fragment inside `run_text`.
    run_offset: usize,
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
            run_offset: 0,
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

    fn with_run_offset(mut self, offset: usize) -> Self {
        self.run_offset = offset;
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
        let run_offset = self.run_offset;
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
                    let up_index =
                        run_offset + preview_index_for_position(&text_layout, event.position);
                    entity.update(cx, |app, cx| {
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
                        if selection_empty
                            && hitbox.is_hovered(window)
                            && let Some(anchor) = drag_anchor_offset
                        {
                            for (range, url) in link_ranges.iter().zip(link_urls.iter()) {
                                if range.contains(&anchor) && range.contains(&up_index) {
                                    cx.open_url(url);
                                    break;
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
                let index = run_offset + preview_index_for_position(&text_layout, event.position);
                entity.update(cx, |app, cx| {
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
                let index = run_offset + preview_index_for_position(&text_layout, event.position);
                entity.update(cx, |app, cx| {
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
                && let ix = run_offset + ix
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
                let index = run_offset + preview_index_for_position(&text_layout, event.position);
                let mut link_url = None;
                for (range, url) in link_ranges.iter().zip(link_urls.iter()) {
                    if range.contains(&index) {
                        link_url = Some(url.clone());
                        break;
                    }
                }
                entity.update(cx, |app, cx| {
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

fn preview_fragment_selection(
    selection: Option<&Range<usize>>,
    fragment: Range<usize>,
) -> Option<Range<usize>> {
    let selection = selection?;
    let start = selection.start.max(fragment.start);
    let end = selection.end.min(fragment.end);
    (start < end).then(|| start - fragment.start..end - fragment.start)
}

fn preview_span_highlight(span: &InlineSpan) -> Option<HighlightStyle> {
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
    if span.link.is_some() {
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

fn preview_math_hit_target(
    block_index: usize,
    run_id: PreviewTextRunId,
    boundary: usize,
    run_text: SharedString,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let down_text = run_text.clone();
    let move_text = run_text.clone();
    let up_text = run_text;
    div()
        .flex_1()
        .h_full()
        .cursor(CursorStyle::IBeam)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseDownEvent, _, cx| {
                app.begin_preview_selection(block_index, run_id, boundary, down_text.clone(), cx);
            }),
        )
        .on_mouse_move(cx.listener(move |app, event: &MouseMoveEvent, _, cx| {
            if event.dragging() {
                app.update_preview_selection_head(
                    block_index,
                    run_id,
                    boundary,
                    move_text.clone(),
                    cx,
                );
            }
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseUpEvent, _, cx| {
                app.update_preview_selection_head(
                    block_index,
                    run_id,
                    boundary,
                    up_text.clone(),
                    cx,
                );
                app.end_preview_selection(cx);
            }),
        )
}

pub(super) fn math_atom_boundary(authored_range: &Range<usize>, trailing_half: bool) -> usize {
    if trailing_half {
        authored_range.end
    } else {
        authored_range.start
    }
}

/// Rendered formula with two atomic hit targets. Pointer positions resolve to
/// the complete authored span's leading or trailing boundary; glyph internals
/// are never exposed as selectable offsets.
fn preview_math_atom(
    app: &MarkionApp,
    image: Arc<MathImage>,
    block_index: usize,
    run_id: PreviewTextRunId,
    authored_range: Range<usize>,
    run_text: SharedString,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let selected = active_preview_run_selection(app, block_index, run_id, run_text.as_ref())
        .is_some_and(|range| range.start < authored_range.end && range.end > authored_range.start);
    let start = math_atom_boundary(&authored_range, false);
    let end = math_atom_boundary(&authored_range, true);
    let metric_height = image.ascent + image.descent;
    div()
        .relative()
        .flex_none()
        .w(image.size.width)
        .h(metric_height)
        .mb(image.descent)
        .when(selected, |atom| atom.bg(rgba(PREVIEW_SELECTION_COLOR)))
        .child(
            img(ImageSource::Render(image.image.clone()))
                .absolute()
                .top_0()
                .left_0()
                .w(image.size.width)
                .h(image.size.height),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .left_0()
                .flex()
                .child(preview_math_hit_target(
                    block_index,
                    run_id,
                    start,
                    run_text.clone(),
                    cx,
                ))
                .child(preview_math_hit_target(
                    block_index,
                    run_id,
                    end,
                    run_text,
                    cx,
                )),
        )
        .into_any_element()
}

/// Mixed prose/math preview path. Text fragments retain their offsets in the
/// single source-backed run, while ready formulas become baseline-aligned,
/// indivisible image atoms. Pending and failed formulas remain exact source.
pub(super) fn rich_text_with_math_element(
    app: &MarkionApp,
    id_prefix: &'static str,
    rich: &RichText,
    block_index: usize,
    run_id: PreviewTextRunId,
    display_scale: f32,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let typography = app.typography_metrics();
    if !rich.spans.iter().any(|span| span.math.is_some()) {
        return rich_text_element(
            app,
            ElementId::from((id_prefix, block_index)),
            rich,
            block_index,
            run_id,
            cx,
        );
    }

    let full_selection = active_preview_run_selection(app, block_index, run_id, &rich.text);
    let run_text = SharedString::from(rich.text.clone());
    let mut children = Vec::new();
    let mut offset = 0usize;
    let mut fragment_index = 0usize;
    for span in &rich.spans {
        let span_range = offset..offset + span.text.len();
        offset = span_range.end;
        if let Some(math) = &span.math {
            match app.math_entry(
                &math.latex,
                math.style,
                typography.math_font_size(math.style),
                1.0,
                display_scale,
                app.palette().text,
            ) {
                MathCacheEntry::Ready(image) => children.push(preview_math_atom(
                    app,
                    image,
                    block_index,
                    run_id,
                    span_range,
                    run_text.clone(),
                    cx,
                )),
                MathCacheEntry::Pending | MathCacheEntry::Error(_) => {
                    let local_len = span.text.len();
                    let mut style = HighlightStyle {
                        color: Some(rgb(PREVIEW_INLINE_CODE_COLOR).into()),
                        background_color: Some(rgba(PREVIEW_INLINE_CODE_BG).into()),
                        ..Default::default()
                    };
                    if matches!(
                        app.math_entry(
                            &math.latex,
                            math.style,
                            typography.math_font_size(math.style),
                            1.0,
                            display_scale,
                            app.palette().text,
                        ),
                        MathCacheEntry::Error(_)
                    ) {
                        style.color = Some(rgb(0xb91c1c).into());
                    }
                    let selection =
                        preview_fragment_selection(full_selection.as_ref(), span_range.clone());
                    children.push(
                        SelectablePreviewText::new(
                            ElementId::from(SharedString::from(format!(
                                "{id_prefix}-{block_index}-{fragment_index}"
                            ))),
                            StyledText::new(SharedString::from(span.text.clone()))
                                .with_highlights(vec![(0..local_len, style)]),
                            block_index,
                            run_id,
                            run_text.clone(),
                            selection,
                            cx.entity(),
                        )
                        .with_run_offset(span_range.start)
                        .into_any_element(),
                    );
                    fragment_index += 1;
                }
            }
            continue;
        }

        let mut fragment_start = span_range.start;
        for fragment in span.text.split_inclusive(char::is_whitespace) {
            if fragment.is_empty() {
                continue;
            }
            let fragment_range = fragment_start..fragment_start + fragment.len();
            let local_range = 0..fragment.len();
            let mut highlights = Vec::new();
            if let Some(style) = preview_span_highlight(span) {
                highlights.push((local_range, style));
            }
            let links = span
                .link
                .as_ref()
                .map_or_else(Vec::new, |_| vec![fragment_range.clone()]);
            let urls = span.link.clone().into_iter().collect();
            let selection =
                preview_fragment_selection(full_selection.as_ref(), fragment_range.clone());
            children.push(
                SelectablePreviewText::new(
                    ElementId::from(SharedString::from(format!(
                        "{id_prefix}-{block_index}-{fragment_index}"
                    ))),
                    StyledText::new(SharedString::from(fragment.to_string()))
                        .with_highlights(highlights),
                    block_index,
                    run_id,
                    run_text.clone(),
                    selection,
                    cx.entity(),
                )
                .with_run_offset(fragment_range.start)
                .with_links(links, urls)
                .into_any_element(),
            );
            fragment_index += 1;
            fragment_start = fragment_range.end;
        }
    }

    div()
        .w_full()
        .flex()
        .flex_wrap()
        .items_end()
        .children(children)
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

/// Visual rows reconcile by stable source lineage rather than byte ranges.
/// The row builder still reads the fresh block slice after the splice, so
/// preserved rows receive current offsets without losing cached heights.
pub(super) fn visual_block_splice(
    old: &[VisualBlock],
    new: &[VisualBlock],
) -> (std::ops::Range<usize>, usize) {
    let old_ids = old.iter().map(|block| block.id).collect::<Vec<_>>();
    let new_ids = new.iter().map(|block| block.id).collect::<Vec<_>>();
    block_splice(&old_ids, &new_ids)
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

pub(super) fn visual_highlight_style(
    inline_style: InlineStyle,
    link: bool,
) -> Option<HighlightStyle> {
    let mut style = HighlightStyle::default();
    let mut styled = false;
    if inline_style.bold {
        style.font_weight = Some(FontWeight::BOLD);
        styled = true;
    }
    if inline_style.italic {
        style.font_style = Some(FontStyle::Italic);
        styled = true;
    }
    if inline_style.strikethrough {
        style.strikethrough = Some(StrikethroughStyle {
            thickness: px(1.),
            color: None,
        });
        styled = true;
    }
    if inline_style.code {
        style.background_color = Some(rgba(PREVIEW_INLINE_CODE_BG).into());
        style.color = Some(rgb(PREVIEW_INLINE_CODE_COLOR).into());
        styled = true;
    }
    if inline_style.highlight {
        style.background_color = Some(rgba(PREVIEW_HIGHLIGHT_BG).into());
        styled = true;
    }
    if inline_style.superscript || inline_style.subscript {
        style.color = Some(rgb(PREVIEW_SUPER_SUB_COLOR).into());
        styled = true;
    }
    if link {
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
    let source_selection = app.active_tab().selected_range.clone();
    let source_cursor = app.active_tab().cursor_offset();
    let marked_range = app.active_tab().marked_range.clone();
    let projection = build_visual_projection_with_marked_range(
        app.active_tab().document.text(),
        block,
        source_selection.clone(),
        source_cursor,
        marked_range.clone(),
    );
    let mut highlights = projection
        .spans
        .iter()
        .filter_map(|span| {
            let style = if span.source {
                Some(HighlightStyle {
                    color: Some(rgb(PREVIEW_INLINE_CODE_COLOR).into()),
                    background_color: Some(rgba(PREVIEW_INLINE_CODE_BG).into()),
                    ..Default::default()
                })
            } else {
                visual_highlight_style(span.style, span.link)
            }?;
            Some((span.display_range.clone(), style))
        })
        .collect::<Vec<_>>();
    if let Some(display_range) = marked_range
        .as_ref()
        .and_then(|range| projection.display_range_for_source_range(range.clone()))
        .filter(|range| !range.is_empty())
    {
        highlights.push((
            display_range,
            HighlightStyle {
                underline: Some(UnderlineStyle {
                    color: Some(rgb(0x2563eb).into()),
                    thickness: px(1.),
                    wavy: false,
                }),
                ..Default::default()
            },
        ));
    }
    #[cfg(test)]
    let test_projection = visual_block_is_focused(app, block).then_some((
        projection.text.clone(),
        projection.revealed_source_ranges.clone(),
    ));
    #[cfg(test)]
    let test_projection_styles = visual_block_is_focused(app, block).then(|| {
        projection
            .spans
            .iter()
            .filter(|span| !span.source)
            .map(|span| span.style)
            .collect()
    });
    VisualEditableText {
        element_id: ElementId::from(("visual-text", block_index)),
        block_index,
        source_island: false,
        text: StyledText::new(SharedString::from(projection.text.clone()))
            .with_highlights(highlights),
        projection,
        source_selection,
        source_cursor,
        marked_range,
        caret_active: visual_block_owns_caret(app, block_index),
        navigation_active: true,
        entity: cx.entity(),
        #[cfg(test)]
        test_projection,
        #[cfg(test)]
        test_projection_styles,
    }
    .into_any_element()
}

fn visual_math_hit_target(boundary: usize, cx: &mut Context<MarkionApp>) -> Div {
    div()
        .flex_1()
        .h_full()
        .cursor(CursorStyle::IBeam)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |app, event: &MouseDownEvent, window, cx| {
                let focus_handle = app.focus_handle.clone();
                window.focus(&focus_handle);
                app.file_tree_query_focused = false;
                app.search_focus = None;
                app.input_marked_len = 0;
                app.active_tab_mut().clear_preview_selection();
                app.active_tab_mut().is_selecting = true;
                if event.modifiers.shift {
                    app.select_to(boundary, cx);
                } else {
                    app.move_to(boundary, cx);
                }
            }),
        )
        .on_mouse_move(cx.listener(move |app, event: &MouseMoveEvent, _, cx| {
            if event.dragging() && app.active_tab().is_selecting {
                app.select_to(boundary, cx);
            }
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseUpEvent, _, _| {
                app.active_tab_mut().is_selecting = false;
            }),
        )
}

fn visual_math_atom(
    app: &MarkionApp,
    image: Arc<MathImage>,
    source_range: Range<usize>,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let selected = {
        let selection = &app.active_tab().selected_range;
        !selection.is_empty()
            && selection.start < source_range.end
            && selection.end > source_range.start
    };
    let metric_height = image.ascent + image.descent;
    div()
        .relative()
        .flex_none()
        .w(image.size.width)
        .h(metric_height)
        .mb(image.descent)
        .when(selected, |atom| atom.bg(rgba(PREVIEW_SELECTION_COLOR)))
        .child(
            img(ImageSource::Render(image.image.clone()))
                .absolute()
                .top_0()
                .left_0()
                .w(image.size.width)
                .h(image.size.height),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .left_0()
                .flex()
                .child(visual_math_hit_target(source_range.start, cx))
                .child(visual_math_hit_target(source_range.end, cx)),
        )
        .into_any_element()
}

fn visual_projection_fragment(
    block_index: usize,
    fragment_index: usize,
    visible: String,
    source_range: Range<usize>,
    style: Option<HighlightStyle>,
    app: &MarkionApp,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let visible_len = visible.len();
    let highlights = style
        .map(|style| vec![(0..visible_len, style)])
        .unwrap_or_default();
    VisualEditableText {
        element_id: ElementId::from(SharedString::from(format!(
            "visual-mixed-{block_index}-{fragment_index}"
        ))),
        block_index,
        source_island: false,
        text: StyledText::new(SharedString::from(visible.clone())).with_highlights(highlights),
        projection: VisualProjection {
            text: visible,
            segments: vec![markion::VisualProjectionSegment {
                display_range: 0..visible_len,
                source_range: source_range.clone(),
            }],
            spans: Vec::new(),
            revealed_source_ranges: Vec::new(),
            source_anchor: source_range.start,
        },
        source_selection: app.active_tab().selected_range.clone(),
        source_cursor: app.active_tab().cursor_offset(),
        marked_range: app.active_tab().marked_range.clone(),
        caret_active: visual_block_owns_caret(app, block_index)
            && (source_range.contains(&app.active_tab().cursor_offset())
                || app.active_tab().cursor_offset() == source_range.end),
        navigation_active: !visual_block_owns_caret(app, block_index)
            || (source_range.contains(&app.active_tab().cursor_offset())
                || app.active_tab().cursor_offset() == source_range.end),
        entity: cx.entity(),
        #[cfg(test)]
        test_projection: None,
        #[cfg(test)]
        test_projection_styles: None,
    }
    .into_any_element()
}

/// Source-backed mixed layout for Visual Edit. A focused formula is already a
/// source piece in `build_visual_projection`; other formulas remain image
/// atoms while adjacent prose keeps exact visible-to-source segments.
pub(super) fn visual_text_with_math_element(
    block: &VisualBlock,
    block_index: usize,
    app: &MarkionApp,
    display_scale: f32,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let typography = app.typography_metrics();
    if !block.editable_runs.iter().any(|run| run.math.is_some()) {
        return visual_text_element(block, block_index, app, cx);
    }

    let projection = build_visual_projection_with_marked_range(
        app.active_tab().document.text(),
        block,
        app.active_tab().selected_range.clone(),
        app.active_tab().cursor_offset(),
        app.active_tab().marked_range.clone(),
    );
    let mut children = Vec::new();
    let mut fragment_index = 0usize;
    for (segment, projected_span) in projection.segments.iter().zip(&projection.spans) {
        let math = (!projected_span.source).then(|| {
            block
                .editable_runs
                .iter()
                .find(|run| run.math.is_some() && run.content_range == segment.source_range)
        });
        if let Some(Some(run)) = math
            && let Some(math) = &run.math
            && let MathCacheEntry::Ready(image) = app.math_entry(
                &math.latex,
                math.style,
                typography.math_font_size(math.style),
                1.0,
                display_scale,
                app.palette().text,
            )
        {
            children.push(visual_math_atom(app, image, math.source_range.clone(), cx));
            continue;
        }

        let visible = &projection.text[segment.display_range.clone()];
        let style = if projected_span.source {
            Some(HighlightStyle {
                color: Some(rgb(PREVIEW_INLINE_CODE_COLOR).into()),
                background_color: Some(rgba(PREVIEW_INLINE_CODE_BG).into()),
                ..Default::default()
            })
        } else {
            visual_highlight_style(projected_span.style, projected_span.link)
        };
        let can_split = visible.len() == segment.source_range.len();
        if can_split {
            let mut local_start = 0usize;
            for fragment in visible.split_inclusive(char::is_whitespace) {
                if fragment.is_empty() {
                    continue;
                }
                let source_start = segment.source_range.start + local_start;
                let source_range = source_start..source_start + fragment.len();
                children.push(visual_projection_fragment(
                    block_index,
                    fragment_index,
                    fragment.to_string(),
                    source_range,
                    style.clone(),
                    app,
                    cx,
                ));
                fragment_index += 1;
                local_start += fragment.len();
            }
        } else {
            children.push(visual_projection_fragment(
                block_index,
                fragment_index,
                visible.to_string(),
                segment.source_range.clone(),
                style,
                app,
                cx,
            ));
            fragment_index += 1;
        }
    }

    div()
        .w_full()
        .flex()
        .flex_wrap()
        .items_end()
        .children(children)
        .into_any_element()
}

pub(super) fn visual_source_island_view(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
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
        .text_size(px(typography.source_island_font_size))
        .line_height(px(typography.source_island_line_height))
        .child(VisualEditableText {
            element_id: ElementId::from(("visual-source-island", block_index)),
            block_index,
            source_island: true,
            text: StyledText::new(SharedString::from(source.clone())),
            projection: VisualProjection {
                text: source.clone(),
                segments: vec![markion::VisualProjectionSegment {
                    display_range: 0..source_len,
                    source_range: block.source_range.clone(),
                }],
                spans: Vec::new(),
                revealed_source_ranges: vec![block.source_range.clone()],
                source_anchor: block.source_range.start,
            },
            source_selection: app.active_tab().selected_range.clone(),
            source_cursor: app.active_tab().cursor_offset(),
            marked_range: app.active_tab().marked_range.clone(),
            caret_active: visual_block_owns_caret(app, block_index),
            navigation_active: true,
            entity: cx.entity(),
            #[cfg(test)]
            test_projection: None,
            #[cfg(test)]
            test_projection_styles: None,
        })
}

/// True when `block_index` is the single visual row that should paint the
/// document caret. Block source ranges can touch (and, for recovered parser
/// overlaps, even intersect), so ownership is resolved through the same
/// first-match lookup that drives caret reveal scrolling.
pub(super) fn visual_block_owns_caret(app: &MarkionApp, block_index: usize) -> bool {
    let tab = app.active_tab();
    visual_block_index_for_offset(
        &tab.visual_list_blocks,
        tab.cursor_offset(),
        tab.document.text().len(),
    ) == Some(block_index)
}

#[cfg(test)]
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

pub(super) fn visual_block_index_for_offset(
    blocks: &[VisualBlock],
    cursor: usize,
    document_len: usize,
) -> Option<usize> {
    blocks.iter().position(|block| {
        visual_source_range_is_focused(&block.source_range, cursor, document_len)
            || (matches!(block.kind, VisualBlockKind::MathBlock { .. })
                && cursor == block.source_range.end)
    })
}

pub(super) fn visual_block_view(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    document_dir: Option<&Path>,
    display_scale: f32,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    let owns_caret = visual_block_owns_caret(app, block_index);
    let always_source = matches!(
        block.source_island,
        Some(
            VisualSourceIslandKind::FrontMatter
                | VisualSourceIslandKind::Code
                | VisualSourceIslandKind::Html
                | VisualSourceIslandKind::Unsupported
        )
    ) || (block.editor.is_none()
        && block
            .editable_runs
            .iter()
            .any(|run| run.conservative_fallback));
    let focused_conservative = owns_caret
        && block.editor.is_none()
        && (block.source_island.is_some() || block.editable_runs.is_empty());
    if focused_conservative || always_source {
        return visual_source_island_view(app, block, block_index, cx);
    }

    match &block.kind {
        VisualBlockKind::Heading { level } => {
            let size = px(typography.heading_font_size((*level).into()));
            div()
                .mt_2()
                .mb_2()
                .text_size(size)
                .font_weight(FontWeight::BOLD)
                .child(visual_text_with_math_element(
                    block,
                    block_index,
                    app,
                    display_scale,
                    cx,
                ))
        }
        VisualBlockKind::Paragraph => div()
            .mb(px(typography.paragraph_spacing))
            .line_height(px(typography.paragraph_line_height))
            .text_size(px(typography.rendered_font_size))
            .child(visual_text_with_math_element(
                block,
                block_index,
                app,
                display_scale,
                cx,
            )),
        VisualBlockKind::ListItem {
            level,
            ordered,
            index,
            checked,
        } => {
            let prefix_revealed = block.block_prefix.as_ref().is_some_and(|prefix| {
                prefix
                    .source_range
                    .contains(&app.active_tab().cursor_offset())
            });
            let marker = if prefix_revealed {
                String::new()
            } else {
                match checked {
                    Some(true) => "☑".to_string(),
                    Some(false) => "☐".to_string(),
                    None if *ordered => format!("{}.", index.unwrap_or(1)),
                    None => match level {
                        1 => "•".to_string(),
                        2 => "◦".to_string(),
                        _ => "▪".to_string(),
                    },
                }
            };
            let visual_level = if prefix_revealed { 1 } else { *level };
            div()
                .mb_1()
                .ml(px((visual_level as f32 - 1.).max(0.) * 18.))
                .text_size(px(typography.rendered_font_size))
                .line_height(px(typography.list_line_height))
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
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(visual_text_with_math_element(
                            block,
                            block_index,
                            app,
                            display_scale,
                            cx,
                        )),
                )
        }
        VisualBlockKind::BlockQuote => div()
            .mb_3()
            .pl_3()
            .border_l_1()
            .border_color(rgb(0x94a3b8))
            .text_color(rgb(0x475569))
            .text_size(px(typography.quote_font_size))
            .line_height(px(typography.quote_line_height))
            .child(visual_text_with_math_element(
                block,
                block_index,
                app,
                display_scale,
                cx,
            )),
        VisualBlockKind::Image {
            alt: image_alt,
            url,
            ..
        } => {
            if let Some(VisualBlockEditor::Image {
                alt,
                destination,
                title,
            }) = block.editor.as_ref()
            {
                return visual_image_editor(
                    app,
                    block_index,
                    block.id,
                    url,
                    image_alt,
                    document_dir,
                    alt,
                    destination,
                    title.as_ref(),
                    cx,
                );
            }
            let offset = block.source_range.start;
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
        VisualBlockKind::Table { rows, .. } => visual_table_view(app, block, block_index, rows, cx),
        VisualBlockKind::Whitespace => {
            let text = app.active_tab().document.text();
            let source_range = block.source_range.clone();
            let line_count = text[source_range.clone()]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                .max(1);
            let row_height = (line_count as f32 * 12.).clamp(12., 72.);

            // Whitespace that owns the caret is rendered above as a source
            // island. This arm is therefore passive layout: keeping the row
            // preserves exact source coverage without turning block spacing
            // into a pointer-created editing surface.
            div()
                .h(px(row_height))
                .debug_selector(|| "visual-whitespace-gap".to_string())
        }
        VisualBlockKind::MathBlock { latex, .. } => {
            if let Some(VisualBlockEditor::Math { payload, .. }) = block.editor.as_ref() {
                return visual_math_editor(
                    app,
                    block,
                    block_index,
                    latex,
                    payload,
                    display_scale,
                    cx,
                );
            }
            match app.math_entry(
                latex,
                MathLayoutStyle::Display,
                typography.display_math_font_size,
                1.0,
                display_scale,
                app.palette().text,
            ) {
                MathCacheEntry::Ready(image) => {
                    let width = image.size.width;
                    div().child(
                        div()
                            .id(ElementId::from(("visual-math-scroll", block_index)))
                            .mb_3()
                            .w_full()
                            .overflow_x_scroll()
                            .child(
                                div()
                                    .w_full()
                                    .min_w(width)
                                    .py_2()
                                    .flex()
                                    .justify_center()
                                    .child(visual_math_atom(
                                        app,
                                        image,
                                        block.source_range.clone(),
                                        cx,
                                    )),
                            ),
                    )
                }
                MathCacheEntry::Pending | MathCacheEntry::Error(_) => {
                    visual_source_island_view(app, block, block_index, cx)
                }
            }
        }
        VisualBlockKind::CodeBlock { language } => {
            if let Some(VisualBlockEditor::Code { payload, .. }) = block.editor.as_ref() {
                visual_code_editor(app, block.id, block_index, language.as_deref(), payload, cx)
            } else {
                visual_source_island_view(app, block, block_index, cx)
            }
        }
        VisualBlockKind::Unsupported => visual_source_island_view(app, block, block_index, cx),
    }
}

fn visual_editor_field_element(
    app: &MarkionApp,
    block_index: usize,
    field: &VisualEditorField,
    element_id: ElementId,
    styled_text: Option<StyledText>,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let source = app.active_tab().document.text();
    let projection = visual_editor_field_projection(source, field);
    let text = projection.text.clone();
    let source_cursor = app.active_tab().cursor_offset();
    let block_owns_caret = visual_block_owns_caret(app, block_index);
    let caret_active = block_owns_caret
        && source_cursor >= field.source_range.start
        && source_cursor <= field.source_range.end;
    let marked_range = app.active_tab().marked_range.clone().filter(|marked| {
        marked.start >= field.source_range.start && marked.end <= field.source_range.end
    });
    #[cfg(test)]
    let test_projection = caret_active.then_some((text.clone(), Vec::new()));
    VisualEditableText {
        element_id,
        block_index,
        source_island: false,
        text: styled_text.unwrap_or_else(|| StyledText::new(SharedString::from(text))),
        projection,
        source_selection: app.active_tab().selected_range.clone(),
        source_cursor,
        marked_range,
        caret_active,
        navigation_active: caret_active || !block_owns_caret,
        entity: cx.entity(),
        #[cfg(test)]
        test_projection,
        #[cfg(test)]
        test_projection_styles: None,
    }
    .into_any_element()
}

pub(super) fn visual_editor_field_projection(
    source: &str,
    field: &VisualEditorField,
) -> VisualProjection {
    let authored = &source[field.source_range.clone()];
    let terminator = match field.kind {
        VisualEditorFieldKind::ImageAlt => Some(']'),
        VisualEditorFieldKind::ImageDestination => Some(')'),
        VisualEditorFieldKind::ImageTitle => field
            .source_range
            .start
            .checked_sub(1)
            .and_then(|offset| source[offset..].chars().next())
            .map(|delimiter| if delimiter == '(' { ')' } else { delimiter }),
        VisualEditorFieldKind::TableCell { .. } => Some('|'),
        VisualEditorFieldKind::CodePayload | VisualEditorFieldKind::MathPayload => None,
    };
    let Some(terminator) = terminator else {
        return VisualProjection {
            text: authored.to_string(),
            segments: (!authored.is_empty())
                .then_some(markion::VisualProjectionSegment {
                    display_range: 0..authored.len(),
                    source_range: field.source_range.clone(),
                })
                .into_iter()
                .collect(),
            spans: Vec::new(),
            revealed_source_ranges: Vec::new(),
            source_anchor: field.source_range.start,
        };
    };

    let mut text = String::with_capacity(authored.len());
    let mut segments = Vec::new();
    let mut chars = authored.char_indices().peekable();
    while let Some((offset, ch)) = chars.next() {
        let source_start = field.source_range.start + offset;
        if ch == '\\'
            && let Some(&(next_offset, next)) = chars.peek()
            && next == terminator
        {
            chars.next();
            let display_start = text.len();
            text.push(next);
            segments.push(markion::VisualProjectionSegment {
                display_range: display_start..text.len(),
                source_range: source_start
                    ..field.source_range.start + next_offset + next.len_utf8(),
            });
            continue;
        }
        let display_start = text.len();
        text.push(ch);
        segments.push(markion::VisualProjectionSegment {
            display_range: display_start..text.len(),
            source_range: source_start..source_start + ch.len_utf8(),
        });
    }
    VisualProjection {
        text,
        segments,
        spans: Vec::new(),
        revealed_source_ranges: Vec::new(),
        source_anchor: field.source_range.start,
    }
}

fn visual_code_editor(
    app: &MarkionApp,
    block_id: VisualBlockId,
    block_index: usize,
    language: Option<&str>,
    payload: &VisualEditorField,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    let code = &app.active_tab().document.text()[payload.source_range.clone()];
    let highlighted = app.highlighted_code(language, code);
    let (styled, _) = code_block_text(&highlighted);
    div()
        .mb_3()
        .p_3()
        .rounded_md()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .font_family("JetBrains Mono")
        .text_size(px(typography.code_font_size))
        .line_height(px(typography.code_line_height))
        .children(language.map(|language| {
            div()
                .mb_2()
                .text_size(px(typography.small_font_size))
                .text_color(rgb(0x93c5fd))
                .child(language.to_string())
        }))
        .child(visual_editor_field_element(
            app,
            block_index,
            payload,
            ElementId::from(("visual-code-payload", block_id.as_u64())),
            Some(styled),
            cx,
        ))
}

fn visual_math_editor(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    latex: &str,
    payload: &VisualEditorField,
    display_scale: f32,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    let presentation = match app.math_entry(
        latex,
        MathLayoutStyle::Display,
        typography.display_math_font_size,
        1.0,
        display_scale,
        app.palette().text,
    ) {
        MathCacheEntry::Ready(image) => div()
            .w_full()
            .py_2()
            .flex()
            .justify_center()
            .child(visual_math_atom(app, image, block.source_range.clone(), cx)),
        MathCacheEntry::Pending => div()
            .py_2()
            .text_color(app.palette().muted)
            .child(app.tr(Msg::MathRendering)),
        MathCacheEntry::Error(error) => div()
            .py_2()
            .text_color(rgb(0xb91c1c))
            .child(app.math_error_message(&error)),
    };
    div()
        .mb_3()
        .border_1()
        .border_color(rgb(0xcbd5e1))
        .rounded_md()
        .overflow_hidden()
        .child(presentation)
        .child(
            div()
                .border_t_1()
                .border_color(rgb(0xe2e8f0))
                .bg(rgb(0xf8fafc))
                .p_2()
                .font_family("JetBrains Mono")
                .text_size(px(typography.code_font_size))
                .line_height(px(typography.code_line_height))
                .child(visual_editor_field_element(
                    app,
                    block_index,
                    payload,
                    ElementId::from(("visual-math-payload", block.id.as_u64())),
                    None,
                    cx,
                )),
        )
}

#[allow(clippy::too_many_arguments)]
fn visual_image_editor(
    app: &MarkionApp,
    block_index: usize,
    block_id: VisualBlockId,
    url: &str,
    image_alt: &str,
    document_dir: Option<&Path>,
    alt: &VisualEditorField,
    destination: &VisualEditorField,
    title: Option<&VisualEditorField>,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let fields = [
        (app.tr(Msg::LabelImageAlt), alt),
        (app.tr(Msg::LabelImageDestination), destination),
    ];
    div()
        .mb_3()
        .border_1()
        .border_color(rgb(0xcbd5e1))
        .rounded_md()
        .overflow_hidden()
        .child(
            div()
                .bg(rgb(0xf8fafc))
                .p_2()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .min_h(px(96.))
                .child(img(preview_image_source(url, document_dir)).max_w_full())
                .children((!image_alt.is_empty()).then(|| {
                    div()
                        .mt_1()
                        .text_size(px(11.))
                        .text_color(rgb(0x64748b))
                        .child(image_alt.to_string())
                })),
        )
        .children(
            fields
                .into_iter()
                .enumerate()
                .map(|(index, (label, field))| {
                    visual_image_field_row(app, block_id, block_index, index, label, field, cx)
                }),
        )
        .children(title.map(|field| {
            visual_image_field_row(
                app,
                block_id,
                block_index,
                2,
                app.tr(Msg::LabelImageTitle),
                field,
                cx,
            )
        }))
}

fn visual_image_field_row(
    app: &MarkionApp,
    block_id: VisualBlockId,
    block_index: usize,
    field_index: usize,
    label: &'static str,
    field: &VisualEditorField,
    cx: &mut Context<MarkionApp>,
) -> Div {
    div()
        .border_t_1()
        .border_color(rgb(0xe2e8f0))
        .px_2()
        .py_1()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .w(px(72.))
                .flex_none()
                .text_size(px(11.))
                .text_color(rgb(0x64748b))
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family("JetBrains Mono")
                .text_size(px(12.))
                .child(visual_editor_field_element(
                    app,
                    block_index,
                    field,
                    ElementId::from((
                        "visual-image-field",
                        block_id.as_u64().wrapping_mul(8) | field_index as u64,
                    )),
                    None,
                    cx,
                )),
        )
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
    block: &VisualBlock,
    block_index: usize,
    rows: &[Vec<String>],
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    let table_offset = block.source_range.start;
    let cells = match block.editor.as_ref() {
        Some(VisualBlockEditor::Table { cells }) => Some(cells),
        _ => None,
    };
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
                    let field = cells.and_then(|cells| {
                        cells
                            .iter()
                            .find(|cell| cell.row == row_index && cell.column == cell_index)
                            .map(|cell| &cell.field)
                    });
                    div()
                        .flex_1()
                        .min_w_0()
                        .p_2()
                        .when(!is_last_cell, |style| {
                            style.border_r_1().border_color(rgb(0xe2e8f0))
                        })
                        .text_size(px(typography.table_font_size))
                        .cursor(CursorStyle::IBeam)
                        .when(field.is_none(), |view| {
                            view.on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |app, _, _, cx| app.move_to(offset, cx)),
                            )
                        })
                        .child(field.map_or_else(
                            || cell.clone().into_any_element(),
                            |field| {
                                visual_editor_field_element(
                                    app,
                                    block_index,
                                    field,
                                    ElementId::from((
                                        "visual-table-cell",
                                        (block.id.as_u64() << 24)
                                            | ((row_index as u64) << 12)
                                            | cell_index as u64,
                                    )),
                                    None,
                                    cx,
                                )
                            },
                        ))
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
    let typography = app.typography_metrics();
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
                    .line_height(px(typography.paragraph_line_height))
                    .text_size(px(typography.rendered_font_size))
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

fn code_block_view(
    app: &MarkionApp,
    language: &Option<String>,
    code: &str,
    block_index: usize,
    show_code_line_numbers: bool,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    let highlighted = app.highlighted_code(language.as_deref(), code);
    let body = div()
        .mb_3()
        .p_3()
        .rounded_md()
        .bg(rgb(0x0f172a))
        .text_color(rgb(0xe2e8f0))
        .font_family("JetBrains Mono")
        .text_size(px(typography.code_font_size))
        .line_height(px(typography.code_line_height))
        .children(language.as_ref().map(|language| {
            div()
                .mb_2()
                .text_size(px(typography.small_font_size))
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

pub(super) fn preview_block_view(
    app: &MarkionApp,
    block: &PreviewBlock,
    block_index: usize,
    document_dir: Option<&Path>,
    show_code_line_numbers: bool,
    display_scale: f32,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    match block {
        PreviewBlock::Heading { level, text, .. } => {
            let size = px(typography.heading_font_size((*level).into()));
            div()
                .mt_2()
                .mb_2()
                .text_size(size)
                .font_weight(gpui::FontWeight::BOLD)
                .child(rich_text_with_math_element(
                    app,
                    "preview-heading",
                    text,
                    block_index,
                    PreviewTextRunId::Body,
                    display_scale,
                    cx,
                ))
        }
        PreviewBlock::Paragraph { text, .. } => div()
            .mb(px(typography.paragraph_spacing))
            .line_height(px(typography.paragraph_line_height))
            .text_size(px(typography.rendered_font_size))
            .child(rich_text_with_math_element(
                app,
                "preview-paragraph",
                text,
                block_index,
                PreviewTextRunId::Body,
                display_scale,
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
                .text_size(px(typography.rendered_font_size))
                .line_height(px(typography.list_line_height))
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
                .child(div().flex_1().min_w_0().child(rich_text_with_math_element(
                    app,
                    "preview-list-item",
                    text,
                    block_index,
                    PreviewTextRunId::Body,
                    display_scale,
                    cx,
                )))
        }
        PreviewBlock::BlockQuote { text, .. } => div()
            .mb_3()
            .pl_3()
            .border_l_1()
            .border_color(rgb(0x94a3b8))
            .text_color(rgb(0x475569))
            .text_size(px(typography.quote_font_size))
            .line_height(px(typography.quote_line_height))
            .child(rich_text_with_math_element(
                app,
                "preview-quote",
                text,
                block_index,
                PreviewTextRunId::Body,
                display_scale,
                cx,
            )),
        PreviewBlock::CodeBlock { language, code, .. } => {
            match app.diagram_entry(language.as_deref(), code) {
                Some(DiagramCacheEntry::Ready(image, size)) => div()
                    .mb_3()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xcbd5e1))
                    .bg(rgb(0xffffff))
                    .overflow_hidden()
                    // The raster is supersampled, and `RenderImage::scale_factor`
                    // can't say so, so an auto-sized element would resolve to the
                    // raw pixel count and draw at double size. Pinning the
                    // intrinsic width and leaving height auto cancels the factor
                    // out. A wider-than-column diagram still reserves its
                    // intrinsic height; see the change's design.md.
                    .child(img(ImageSource::Render(image)).w(size.width).max_w_full()),
                Some(DiagramCacheEntry::Pending) => div()
                    .mb_3()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xbfdbfe))
                    .bg(rgb(0xeff6ff))
                    .child(
                        div()
                            .mb_2()
                            .text_color(rgb(0x1d4ed8))
                            .child(t(app.language, Msg::DiagramLoading)),
                    )
                    .child(code_block_view(
                        app,
                        language,
                        code,
                        block_index,
                        show_code_line_numbers,
                        cx,
                    )),
                Some(DiagramCacheEntry::Error(error)) => div()
                    .mb_3()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xfca5a5))
                    .bg(rgb(0xfef2f2))
                    .child(
                        div()
                            .mb_2()
                            .text_color(rgb(0xb91c1c))
                            .child(app.diagram_error_message(&error)),
                    )
                    .child(code_block_view(
                        app,
                        language,
                        code,
                        block_index,
                        show_code_line_numbers,
                        cx,
                    )),
                None => {
                    code_block_view(app, language, code, block_index, show_code_line_numbers, cx)
                }
            }
        }
        PreviewBlock::MathBlock {
            latex,
            authored,
            error,
            ..
        } => {
            let entry = app.math_entry(
                latex,
                MathLayoutStyle::Display,
                typography.display_math_font_size,
                1.0,
                display_scale,
                app.palette().text,
            );
            if error.is_none()
                && let MathCacheEntry::Ready(image) = &entry
            {
                let image = image.clone();
                let width = image.size.width;
                div().child(
                    div()
                        .id(ElementId::from(("preview-math-scroll", block_index)))
                        .mb_3()
                        .w_full()
                        .overflow_x_scroll()
                        .child(
                            div()
                                .w_full()
                                .min_w(width)
                                .py_2()
                                .flex()
                                .justify_center()
                                .child(preview_math_atom(
                                    app,
                                    image,
                                    block_index,
                                    PreviewTextRunId::MathLatex,
                                    0..authored.len(),
                                    SharedString::from(authored.clone()),
                                    cx,
                                )),
                        ),
                )
            } else {
                let (label, detail) = match entry {
                    MathCacheEntry::Pending if error.is_none() => {
                        (t(app.language, Msg::MathRendering), None)
                    }
                    MathCacheEntry::Error(renderer_error) => (
                        app.math_error_message(&renderer_error),
                        Some(renderer_error.to_string()),
                    ),
                    _ => (t(app.language, Msg::MathInvalid), error.clone()),
                };
                div().child(
                    div()
                        .id(ElementId::from(("preview-math-fallback", block_index)))
                        .mb_3()
                        .p_3()
                        .max_h(px(240.))
                        .overflow_y_scroll()
                        .rounded_md()
                        .border_1()
                        .border_color(if error.is_some() {
                            rgb(0xfca5a5)
                        } else {
                            rgb(0xcbd5e1)
                        })
                        .bg(if error.is_some() {
                            rgb(0xfef2f2)
                        } else {
                            rgb(0xf8fafc)
                        })
                        .child(div().mb_2().text_size(px(12.)).child(label))
                        .when_some(detail, |panel, detail| {
                            panel.child(
                                div()
                                    .mb_2()
                                    .text_size(px(11.))
                                    .text_color(rgb(0xb91c1c))
                                    .child(detail),
                            )
                        })
                        .child(selectable_plain_text(
                            app,
                            ElementId::from(("preview-math-source", block_index)),
                            StyledText::new(SharedString::from(authored.clone())),
                            authored.clone(),
                            block_index,
                            PreviewTextRunId::MathLatex,
                            cx,
                        )),
                )
            }
        }
        PreviewBlock::Html { html, .. } => {
            html_preview_block_view(app, html, block_index, document_dir, cx)
        }
        PreviewBlock::Image { url, .. } => div()
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
            ),
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
                                .text_size(px(typography.table_font_size))
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
        return remote_image_request_url(url).to_string().into();
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

/// Returns the request URL for a remote image without its client-side fragment.
/// HTTP request targets cannot contain URI fragments; GPUI otherwise treats such
/// sources as embedded assets rather than loading them over the network.
pub(super) fn remote_image_request_url(url: &str) -> &str {
    if is_http_resource(url) {
        url.split_once('#')
            .map_or(url, |(request_url, _fragment)| request_url)
    } else {
        url
    }
}

fn is_http_resource(url: &str) -> bool {
    url.get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
        || url
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
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
