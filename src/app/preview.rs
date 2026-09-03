use super::*;

pub(super) fn read_mode_preview_is_constrained(
    view_mode: ViewMode,
    preview_adaptive_width: bool,
) -> bool {
    matches!(view_mode, ViewMode::Read | ViewMode::VisualEdit) && !preview_adaptive_width
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

/// Whether source-mapped scroll coupling should run this frame. It is active
/// only in Split Preview, the sole mode where both panes are visible.
pub(super) fn sync_scroll_is_active(view_mode: ViewMode, sync_scroll: bool) -> bool {
    matches!(view_mode, ViewMode::Split) && sync_scroll
}

pub(super) fn sync_scroll_mapping_is_current(
    document_version: u64,
    source_layout_key: Option<SourceLayoutKey>,
    preview_reflects_version: Option<u64>,
    has_preview_blocks: bool,
) -> bool {
    source_layout_key.is_some_and(|key| key.version == document_version)
        && preview_reflects_version == Some(document_version)
        && has_preview_blocks
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum PreviewScrollAnchor {
    Start,
    End,
    Block { item_ix: usize },
}

pub(super) const SYNC_SCROLL_PIXEL_EPSILON: f32 = 1.0;

/// Return the preview-list row for the exact outline heading source offset.
///
/// Outline headings and preview heading blocks are derived from the same
/// Markdown source positions. Matching that identity avoids ambiguities from
/// duplicate titles or generated anchors and keeps this click-time lookup free
/// of parsing or additional cached state.
pub(super) fn preview_heading_index_for_source_offset(
    blocks: &[PreviewBlock],
    heading_offset: usize,
) -> Option<usize> {
    blocks.iter().position(|block| {
        matches!(
            block,
            PreviewBlock::Heading { source_range, .. }
                if source_range.start == heading_offset
        )
    })
}

/// Find the preview row that owns `source_offset`. Gaps with no rendered row
/// collapse to the following row boundary; leading/trailing gaps become the
/// document boundaries.
pub(super) fn preview_anchor_for_source_offset(
    blocks: &[PreviewBlock],
    source_offset: usize,
    document_len: usize,
) -> Option<PreviewScrollAnchor> {
    if blocks.is_empty() {
        return None;
    }
    if source_offset == 0 {
        return Some(PreviewScrollAnchor::Start);
    }
    if source_offset >= document_len {
        return Some(PreviewScrollAnchor::End);
    }
    if source_offset < blocks[0].source_range().start {
        return Some(PreviewScrollAnchor::Start);
    }
    let item_ix = blocks.partition_point(|block| block.source_range().end <= source_offset);
    if item_ix >= blocks.len() {
        Some(PreviewScrollAnchor::End)
    } else {
        Some(PreviewScrollAnchor::Block { item_ix })
    }
}

pub(super) fn sync_interval_progress(value: f32, start: f32, end: f32) -> f32 {
    let extent = end - start;
    if extent <= SYNC_SCROLL_PIXEL_EPSILON {
        return 0.;
    }
    ((value - start) / extent).clamp(0., 1.)
}

pub(super) fn sync_interpolate(start: f32, end: f32, progress: f32) -> f32 {
    start + (end - start) * progress.clamp(0., 1.)
}

fn preview_position_changed(left: SyncPreviewPosition, right: SyncPreviewPosition) -> bool {
    left.item_ix != right.item_ix
        || (left.offset_in_item - right.offset_in_item).abs() > SYNC_SCROLL_PIXEL_EPSILON
}

/// Resolve the current driver while consuming one expected follower write.
/// Explicit interaction wins; a deferred driver wins over raw geometry drift;
/// otherwise exactly one changed pane may drive.
pub(super) fn select_sync_scroll_driver(
    state: &mut SyncScrollState,
    editor_offset: f32,
    preview_position: SyncPreviewPosition,
) -> Option<PaneScrollTarget> {
    let explicit = state.driver_hint.take();
    let mut suppress_editor = false;
    let mut suppress_preview = false;
    if let Some(expected) = state.expected_follower.take() {
        state.expected_follower_retried = false;
        match expected {
            ExpectedSyncFollower::Editor(expected_offset) => {
                suppress_editor = explicit != Some(PaneScrollTarget::Editor)
                    || (editor_offset - expected_offset).abs() <= SYNC_SCROLL_PIXEL_EPSILON;
            }
            ExpectedSyncFollower::Preview(expected_position) => {
                suppress_preview = explicit != Some(PaneScrollTarget::Preview)
                    || !preview_position_changed(preview_position, expected_position);
            }
        }
    }

    let editor_changed = state
        .last_editor_offset
        .is_some_and(|previous| (previous - editor_offset).abs() > SYNC_SCROLL_PIXEL_EPSILON)
        && !suppress_editor;
    let preview_changed = state
        .last_preview_position
        .is_some_and(|previous| preview_position_changed(previous, preview_position))
        && !suppress_preview;
    let deferred = if explicit.is_none() {
        state.deferred_driver.take()
    } else {
        state.deferred_driver = None;
        None
    };
    let driver = explicit
        .or(deferred)
        .or(match (editor_changed, preview_changed) {
            (true, false) => Some(PaneScrollTarget::Editor),
            (false, true) => Some(PaneScrollTarget::Preview),
            _ => None,
        });
    state.last_editor_offset = Some(editor_offset);
    state.last_preview_position = Some(preview_position);
    driver
}

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
            | PreviewBlock::FootnoteDefinition { text, .. },
            PreviewTextRunId::Body,
        ) => Some(text.text.clone()),
        (PreviewBlock::BlockQuote { children, .. }, PreviewTextRunId::QuoteChild(index)) => {
            children.get(index).map(|child| child.plain_text())
        }
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
        (PreviewBlock::Table { rows, .. }, PreviewTextRunId::TableCell { row, col }) => rows
            .get(row)
            .and_then(|r| r.get(col))
            .map(|cell| cell.text.clone()),
        _ => None,
    }
}

/// Document-order list of selectable runs for a preview block.
pub(super) fn preview_block_runs(block: &PreviewBlock) -> Vec<PreviewTextRunId> {
    match block {
        PreviewBlock::Heading { .. }
        | PreviewBlock::Paragraph { .. }
        | PreviewBlock::ListItem { .. }
        | PreviewBlock::FootnoteDefinition { .. } => vec![PreviewTextRunId::Body],
        PreviewBlock::BlockQuote { children, .. } => children
            .iter()
            .enumerate()
            .filter(|(_, child)| !child.plain_text().is_empty())
            .map(|(index, _)| PreviewTextRunId::QuoteChild(index))
            .collect(),
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

/// Canonical rendered-text runs searched in Read mode. This deliberately
/// differs from selection runs: authored math is selectable as a fallback,
/// but it is a non-text rendered atom and therefore is not searchable. Code
/// uses one canonical body run so line-number presentation cannot duplicate
/// results.
pub(super) fn preview_search_runs(block: &PreviewBlock) -> Vec<PreviewTextRunId> {
    preview_block_runs(block)
        .into_iter()
        .filter(|run_id| {
            !matches!(
                run_id,
                PreviewTextRunId::MathLatex | PreviewTextRunId::CodeLine(_)
            )
        })
        .collect()
}

pub(super) fn preview_search_matches(
    blocks: &[PreviewBlock],
    pattern: &SearchPattern,
) -> Vec<PreviewSearchMatch> {
    let mut matches = Vec::new();
    for (block_index, block) in blocks.iter().enumerate() {
        for run_id in preview_search_runs(block) {
            let Some(text) = preview_run_plain_text(block, run_id) else {
                continue;
            };
            matches.extend(pattern.find_ranges(&text).into_iter().map(|range| {
                PreviewSearchMatch {
                    block_index,
                    run_id,
                    range,
                }
            }));
        }
    }
    matches
}

/// Search ranges to paint for a displayed preview run. Canonical code-body
/// ranges are translated into line-local ranges when line-number mode splits
/// the block into separate shaped lines.
pub(super) fn active_preview_search_ranges(
    app: &MarkionApp,
    block_index: usize,
    run_id: PreviewTextRunId,
    run_text: &str,
) -> Vec<(Range<usize>, bool)> {
    if !app.search_visible || !matches!(app.view_mode, ViewMode::Read) {
        return Vec::new();
    }
    let current = app.current_search_index;
    let mut ranges = Vec::new();
    for (index, target) in app.search_matches.iter().enumerate() {
        let SearchTarget::ReadPreview(target) = target else {
            continue;
        };
        if target.block_index != block_index {
            continue;
        }
        if target.run_id == run_id {
            ranges.push((
                normalize_preview_selection_range(run_text, target.range.clone()),
                current == Some(index),
            ));
            continue;
        }
        if target.run_id == PreviewTextRunId::CodeBody
            && let PreviewTextRunId::CodeLine(line_index) = run_id
        {
            let Some(block) = app.active_tab().preview_list_blocks.get(block_index) else {
                continue;
            };
            let Some(code) = preview_run_plain_text(block, PreviewTextRunId::CodeBody) else {
                continue;
            };
            let mut line_start = 0usize;
            for (code_line_index, line) in code.split('\n').enumerate() {
                let line_end = line_start + line.len();
                if code_line_index == line_index {
                    let start = target.range.start.max(line_start);
                    let end = target.range.end.min(line_end);
                    if start <= end
                        && target.range.start <= line_end
                        && target.range.end >= line_start
                    {
                        ranges.push((start - line_start..end - line_start, current == Some(index)));
                    }
                    break;
                }
                line_start = line_end + 1;
            }
        }
    }
    ranges.retain(|(range, _)| range.start <= range.end && range.end <= run_text.len());
    ranges
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
        self.app.update(cx, |app, cx| {
            let inset = px(app.typography_metrics().preview_row_line_height);
            let tab = app.active_tab_mut();
            tab.visual_input_bounds = Some(bounds);
            if tab.visual_caret_follow_frames == 0 {
                return;
            }
            tab.visual_caret_follow_frames = tab.visual_caret_follow_frames.saturating_sub(1);
            let Some(caret) = tab.visual_caret_bounds else {
                return;
            };
            let list = tab.visual_list.clone();
            if follow_visual_caret_in_list(&list, caret, inset) {
                cx.notify();
            }
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
    /// When set, this element is a whitespace insertion row: it fills the
    /// parent, paints the caret at `caret_shift` from the top, and maps
    /// clicks by Y onto the covered source newlines.
    whitespace_caret: Option<WhitespaceCaretLayout>,
    #[cfg(test)]
    test_projection: Option<(String, Vec<Range<usize>>)>,
    #[cfg(test)]
    test_projection_styles: Option<Vec<InlineStyle>>,
}

/// Geometry for a caret-owning Visual Edit whitespace row.
#[derive(Clone)]
struct WhitespaceCaretLayout {
    caret_shift: Pixels,
    source_range: Range<usize>,
    line_height: f32,
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
        if self.whitespace_caret.is_some() {
            let mut style = Style::default();
            style.size.width = gpui::relative(1.).into();
            style.size.height = gpui::relative(1.).into();
            return (window.request_layout(style, [], cx), ());
        }
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
        if self.whitespace_caret.is_none() {
            self.text
                .prepaint(None, inspector_id, bounds, state, window, cx);
        }
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
        let whitespace_caret = self.whitespace_caret.clone();
        let is_whitespace_row = whitespace_caret.is_some();
        let layout = (!is_whitespace_row).then(|| self.text.layout().clone());
        let affinity = self
            .entity
            .read(cx)
            .active_tab()
            .current_visual_caret_affinity();
        let caret_bounds = if let Some(whitespace) = whitespace_caret.as_ref() {
            self.caret_active.then(|| {
                Bounds::new(
                    point(bounds.origin.x, bounds.origin.y + whitespace.caret_shift),
                    size(px(2.), px(whitespace.line_height)),
                )
            })
        } else {
            self.caret_active
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
                .and_then(|index| layout.as_ref()?.position_for_index(index))
                .map(|position| {
                    let line_height = layout
                        .as_ref()
                        .map(|layout| layout.line_height())
                        .unwrap_or(px(self
                            .entity
                            .read(cx)
                            .typography_metrics()
                            .paragraph_line_height));
                    Bounds::new(position, size(px(2.), line_height))
                })
        };
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
        } else if let Some(layout) = layout.as_ref() {
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
                    for quad in preview_selection_paint_quads(layout, visible_start..visible_end) {
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
        if let (Some(marked_range), Some(layout)) = (self.marked_range.clone(), layout.as_ref()) {
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
                for quad in preview_selection_paint_quads(layout, display_start..display_end) {
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
            if let Some(layout) = layout.as_ref() {
                let navigation_snapshot = visual_navigation_snapshot(
                    document_version,
                    self.block_index,
                    self.source_selection.clone(),
                    marked_range,
                    self.source_island,
                    &self.projection,
                    layout,
                );
                self.entity.update(cx, |app, cx| {
                    app.active_tab_mut()
                        .register_visual_navigation_snapshot(navigation_snapshot);
                    app.complete_pending_visual_navigation(cx);
                });
            }
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
        let whitespace_click = whitespace_caret.clone();
        let row_top = bounds.top();
        let hitbox_for_down = hitbox.clone();
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble
                || event.button != MouseButton::Left
                || !hitbox_for_down.is_hovered(window)
            {
                return;
            }
            let (source, affinity) = if let Some(whitespace) = whitespace_click.as_ref() {
                let text = entity.read(cx).active_tab().document.text().to_string();
                (
                    whitespace_source_at_y(
                        whitespace.source_range.clone(),
                        event.position.y - row_top,
                        &text,
                        whitespace.line_height,
                    ),
                    None,
                )
            } else if let Some(text_layout) = text_layout.as_ref() {
                let visible = preview_index_for_position(text_layout, event.position);
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
                (
                    candidates.resolve(affinity.unwrap_or(VisualCaretAffinity::Downstream)),
                    affinity,
                )
            } else {
                return;
            };
            let focus_handle = entity.read(cx).focus_handle.clone();
            window.focus(&focus_handle);
            entity.update(cx, |app, cx| {
                app.file_tree_query_focused = false;
                app.search_focus = None;
                app.search_control_focus = None;
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
        let whitespace_drag = whitespace_caret;
        let row_top = bounds.top();
        let hitbox_for_move = hitbox.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble
                || !event.dragging()
                || !hitbox_for_move.is_hovered(window)
                || !entity.read(cx).active_tab().is_selecting
            {
                return;
            }
            let source = if let Some(whitespace) = whitespace_drag.as_ref() {
                let text = entity.read(cx).active_tab().document.text().to_string();
                whitespace_source_at_y(
                    whitespace.source_range.clone(),
                    event.position.y - row_top,
                    &text,
                    whitespace.line_height,
                )
            } else if let Some(text_layout) = text_layout.as_ref() {
                let visible = preview_index_for_position(text_layout, event.position);
                let candidates = projection.boundary_candidates(visible);
                candidates.resolve(VisualCaretAffinity::Downstream)
            } else {
                return;
            };
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
        if !is_whitespace_row {
            self.text
                .paint(None, inspector_id, bounds, &mut (), &mut (), window, cx);
        }
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
    search_ranges: Vec<(Range<usize>, bool)>,
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
            search_ranges: Vec::new(),
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

    fn with_search_ranges(mut self, ranges: Vec<(Range<usize>, bool)>) -> Self {
        self.search_ranges = ranges;
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
        let palette = self.entity.read(cx).palette();
        for (range, current) in self.search_ranges.clone() {
            let color = if current {
                palette.search_current
            } else {
                palette.search_match
            };
            for quad in preview_selection_paint_quads(&text_layout, range) {
                window.paint_quad(fill(quad.bounds, color));
            }
        }
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
        if span.style.underline {
            style.underline = Some(UnderlineStyle {
                thickness: px(1.),
                color: None,
                wavy: false,
            });
            styled = true;
        }
        if let Some(color) = span.style.color {
            style.color = Some(rgb(color).into());
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
    let search_ranges = active_preview_search_ranges(app, block_index, run_id, &rich.text);
    SelectablePreviewText::new(
        id,
        styled_text,
        block_index,
        run_id,
        rich.text.clone(),
        selection,
        cx.entity().clone(),
    )
    .with_search_ranges(search_ranges)
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

fn preview_fragment_search_ranges(
    ranges: &[(Range<usize>, bool)],
    fragment: Range<usize>,
) -> Vec<(Range<usize>, bool)> {
    ranges
        .iter()
        .filter_map(|(range, current)| {
            let start = range.start.max(fragment.start);
            let end = range.end.min(fragment.end);
            (start <= end && range.start <= fragment.end && range.end >= fragment.start)
                .then_some((start - fragment.start..end - fragment.start, *current))
        })
        .collect()
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
    if span.style.underline {
        style.underline = Some(UnderlineStyle {
            thickness: px(1.),
            color: None,
            wavy: false,
        });
        styled = true;
    }
    if let Some(color) = span.style.color {
        style.color = Some(rgb(color).into());
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

/// Bottom margin that places a math atom's baseline on the surrounding text
/// baseline inside an `items_end` flex row.
///
/// GPUI measured text does not report baselines to taffy, so `items_baseline`
/// cannot align image atoms with prose. With `items_end`, both boxes share a
/// bottom edge; this margin lifts the formula so `ascent` from its top meets
/// the text baseline (mirroring HTML `vertical-align: -descent` relative to
/// GPUI's half-leading baseline offset).
pub(super) fn inline_math_baseline_margin_from_metrics(
    line_height: Pixels,
    font_ascent: Pixels,
    font_descent: Pixels,
    math_descent: Pixels,
) -> Pixels {
    let content = font_ascent + font_descent;
    let padding_top = ((line_height - content) / 2.).max(px(0.));
    let text_baseline_from_bottom = line_height - padding_top - font_ascent;
    text_baseline_from_bottom - math_descent
}

fn inline_math_baseline_margin(
    cx: &App,
    font_family: &SharedString,
    font_size: f32,
    line_height: f32,
    math_descent: Pixels,
) -> Pixels {
    let font_id = cx.text_system().resolve_font(&font(font_family.clone()));
    let font_size = px(font_size);
    let line_height = px(line_height);
    let font_ascent = cx.text_system().ascent(font_id, font_size);
    let font_descent = cx.text_system().descent(font_id, font_size);
    inline_math_baseline_margin_from_metrics(line_height, font_ascent, font_descent, math_descent)
}

/// GPUI's default text line-height when a block does not set one explicitly
/// (`relative(phi)` ≈ 1.618 × font size).
fn default_text_line_height(font_size: f32) -> f32 {
    font_size * 1.618_034
}

fn visual_inline_text_metrics(
    block: &VisualBlock,
    typography: DocumentTypographyMetrics,
) -> (f32, f32) {
    if block.quote_context.is_some() {
        return (typography.quote_font_size, typography.quote_line_height);
    }
    match &block.kind {
        VisualBlockKind::Heading { level } => {
            let size = typography.heading_font_size((*level).into());
            (size, default_text_line_height(size))
        }
        VisualBlockKind::BlockQuote => (typography.quote_font_size, typography.quote_line_height),
        VisualBlockKind::ListItem { .. } => {
            (typography.rendered_font_size, typography.list_line_height)
        }
        _ => (
            typography.rendered_font_size,
            typography.paragraph_line_height,
        ),
    }
}

/// Rendered formula with two atomic hit targets. Pointer positions resolve to
/// the complete authored span's leading or trailing boundary; glyph internals
/// are never exposed as selectable offsets.
///
/// When `inline_metrics` is `Some((font_size, line_height))`, the atom is
/// baseline-compensated for an `items_end` prose row. Display/block callers
/// pass `None`.
fn preview_math_atom(
    app: &MarkionApp,
    image: Arc<MathImage>,
    block_index: usize,
    run_id: PreviewTextRunId,
    authored_range: Range<usize>,
    run_text: SharedString,
    inline_metrics: Option<(f32, f32)>,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let selected = active_preview_run_selection(app, block_index, run_id, run_text.as_ref())
        .is_some_and(|range| range.start < authored_range.end && range.end > authored_range.start);
    let start = math_atom_boundary(&authored_range, false);
    let end = math_atom_boundary(&authored_range, true);
    let metric_height = image.ascent + image.descent;
    let baseline_margin = inline_metrics.map_or(px(0.), |(font_size, line_height)| {
        inline_math_baseline_margin(
            cx,
            &app.resolved_font_families.rendered,
            font_size,
            line_height,
            image.descent,
        )
    });
    div()
        .relative()
        .flex_none()
        .w(image.size.width)
        .h(metric_height)
        .mb(baseline_margin)
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
    font_size: f32,
    line_height: f32,
    document_dir: Option<&Path>,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let typography = app.typography_metrics();
    if !rich
        .spans
        .iter()
        .any(|span| span.math.is_some() || span.image.is_some())
    {
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
    let full_search_ranges = active_preview_search_ranges(app, block_index, run_id, &rich.text);
    let run_text = SharedString::from(rich.text.clone());
    let inline_metrics = Some((font_size, line_height));
    let mut children = Vec::new();
    let mut offset = 0usize;
    let mut fragment_index = 0usize;
    for span in &rich.spans {
        let span_range = offset..offset + span.text.len();
        offset = span_range.end;
        if let Some(image) = &span.image {
            children.push(
                preview_inline_image_view(app, &image.url, &image.alt, document_dir, None, None)
                    .into_any_element(),
            );
            fragment_index += 1;
            continue;
        }
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
                    inline_metrics,
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
                    let search_ranges =
                        preview_fragment_search_ranges(&full_search_ranges, span_range.clone());
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
                        .with_search_ranges(search_ranges)
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
            let search_ranges =
                preview_fragment_search_ranges(&full_search_ranges, fragment_range.clone());
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
                .with_search_ranges(search_ranges)
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
    let search_ranges = active_preview_search_ranges(app, block_index, run_id, plain.as_ref());
    SelectablePreviewText::new(
        id,
        styled,
        block_index,
        run_id,
        plain,
        selection,
        cx.entity().clone(),
    )
    .with_search_ranges(search_ranges)
    .into_any_element()
}

/// One shaped line of highlighted code (used when line numbers are shown).
pub(super) fn code_line_text(
    line: &[HighlightedSpan],
    palette: &CodePalette,
) -> (StyledText, String) {
    let mut text = String::new();
    let mut highlights = Vec::new();
    for span in line {
        let start = text.len();
        text.push_str(&span.text);
        if span.kind != HighlightKind::Plain {
            highlights.push((
                start..text.len(),
                HighlightStyle {
                    color: Some(palette.token_color(span.kind).into()),
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
pub(super) fn code_block_text(
    lines: &[Vec<HighlightedSpan>],
    palette: &CodePalette,
) -> (StyledText, String) {
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
                        color: Some(palette.token_color(span.kind).into()),
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
/// Height-mutable rows (whitespace) additionally compare their clamped line
/// count — identity alone would let a grown or shrunk whitespace row keep
/// a stale cached list height and under-report the scroll extent. Pixel
/// height is a render-time `paragraph_line_height` multiplier; splice only
/// needs the clamped line count so growth past the sanity bound (which no
/// longer changes the rendered height) also stops forcing re-measures.
pub(super) fn visual_block_splice(
    old: &[VisualBlock],
    new: &[VisualBlock],
) -> (std::ops::Range<usize>, usize) {
    let row_identity = |block: &VisualBlock| {
        (
            block.id,
            block
                .height_signature
                .map(|lines| whitespace_clamped_line_count(lines as usize)),
        )
    };
    let old_ids = old.iter().map(row_identity).collect::<Vec<_>>();
    let new_ids = new.iter().map(row_identity).collect::<Vec<_>>();
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
    if inline_style.underline {
        style.underline = Some(UnderlineStyle {
            thickness: px(1.),
            color: None,
            wavy: false,
        });
        styled = true;
    }
    if let Some(color) = inline_style.color {
        style.color = Some(rgb(color).into());
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

fn overlay_highlight_style(mut base: HighlightStyle, overlay: HighlightStyle) -> HighlightStyle {
    if overlay.color.is_some() {
        base.color = overlay.color;
    }
    if overlay.font_weight.is_some() {
        base.font_weight = overlay.font_weight;
    }
    if overlay.font_style.is_some() {
        base.font_style = overlay.font_style;
    }
    if overlay.background_color.is_some() {
        base.background_color = overlay.background_color;
    }
    if overlay.underline.is_some() {
        base.underline = overlay.underline;
    }
    if overlay.strikethrough.is_some() {
        base.strikethrough = overlay.strikethrough;
    }
    if overlay.fade_out.is_some() {
        base.fade_out = overlay.fade_out;
    }
    base
}

/// Adds one visual overlay while preserving StyledText's contract: highlight
/// ranges must be UTF-8 boundaries, sorted, and non-overlapping. IME marked
/// text is an overlay on the projection's existing inline styles, so simply
/// appending it after later bold/link runs corrupts the resulting TextRun
/// lengths on Windows DirectWrite.
fn overlay_visual_highlight(
    text: &str,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
    overlay_range: Range<usize>,
    overlay_style: HighlightStyle,
) -> Vec<(Range<usize>, HighlightStyle)> {
    let mut highlights = highlights
        .into_iter()
        .filter(|(range, _)| !range.is_empty() && text.get(range.clone()).is_some())
        .collect::<Vec<_>>();
    highlights.sort_by_key(|(range, _)| (range.start, range.end));
    if overlay_range.is_empty() || text.get(overlay_range.clone()).is_none() {
        return highlights;
    }

    let mut boundaries = vec![overlay_range.start, overlay_range.end];
    boundaries.extend(
        highlights
            .iter()
            .flat_map(|(range, _)| [range.start, range.end]),
    );
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut result: Vec<(Range<usize>, HighlightStyle)> = Vec::new();
    for pair in boundaries.windows(2) {
        let range = pair[0]..pair[1];
        if range.is_empty() {
            continue;
        }
        let base = highlights.iter().find_map(|(highlighted, style)| {
            (highlighted.start <= range.start && highlighted.end >= range.end).then_some(*style)
        });
        let overlaid = overlay_range.start <= range.start && overlay_range.end >= range.end;
        let style = match (base, overlaid) {
            (Some(base), true) => overlay_highlight_style(base, overlay_style),
            (Some(base), false) => base,
            (None, true) => overlay_style,
            (None, false) => continue,
        };
        if let Some((previous, previous_style)) = result.last_mut()
            && previous.end == range.start
            && *previous_style == style
        {
            previous.end = range.end;
        } else {
            result.push((range, style));
        }
    }
    result
}

pub(super) fn visual_projection_highlights(
    projection: &VisualProjection,
    marked_range: Option<&Range<usize>>,
) -> Vec<(Range<usize>, HighlightStyle)> {
    let highlights = projection
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
    let Some(display_range) = marked_range
        .and_then(|range| projection.display_range_for_source_range(range.clone()))
        .filter(|range| !range.is_empty())
    else {
        return highlights;
    };
    overlay_visual_highlight(
        &projection.text,
        highlights,
        display_range,
        HighlightStyle {
            underline: Some(UnderlineStyle {
                color: Some(rgb(0x2563eb).into()),
                thickness: px(1.),
                wavy: false,
            }),
            ..Default::default()
        },
    )
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
    let mut highlights = visual_projection_highlights(&projection, marked_range.as_ref());
    if app.search_visible {
        for (index, target) in app.search_matches.iter().enumerate() {
            let SearchTarget::Source(found) = target else {
                continue;
            };
            let Some(display_range) = projection
                .display_range_for_source_range(found.range.clone())
                .filter(|range| !range.is_empty())
            else {
                continue;
            };
            highlights = overlay_visual_highlight(
                &projection.text,
                highlights,
                display_range,
                HighlightStyle {
                    background_color: Some(
                        if app.current_search_index == Some(index) {
                            app.palette().search_current
                        } else {
                            app.palette().search_match
                        }
                        .into(),
                    ),
                    ..Default::default()
                },
            );
        }
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
        whitespace_caret: None,
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
                app.search_control_focus = None;
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
    inline_metrics: Option<(f32, f32)>,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let selected = {
        let selection = &app.active_tab().selected_range;
        !selection.is_empty()
            && selection.start < source_range.end
            && selection.end > source_range.start
    };
    let metric_height = image.ascent + image.descent;
    let baseline_margin = inline_metrics.map_or(px(0.), |(font_size, line_height)| {
        inline_math_baseline_margin(
            cx,
            &app.resolved_font_families.rendered,
            font_size,
            line_height,
            image.descent,
        )
    });
    div()
        .relative()
        .flex_none()
        .w(image.size.width)
        .h(metric_height)
        .mb(baseline_margin)
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

/// Test-only counter of inline HTML image atoms built during rendering.
#[cfg(test)]
pub(super) static VISUAL_HTML_IMAGE_ATOM_BUILDS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Inline raw-HTML `<img>` atom inside Visual Edit prose. Mirrors the math
/// atom: the loaded image (or a compact alt/URL chip while pending or on
/// error) with a selection highlight and start/end hit targets that place the
/// caret at the authored tag's byte boundaries.
fn visual_html_image_atom(
    app: &MarkionApp,
    image: &VisualHtmlImage,
    source_range: Range<usize>,
    document_dir: Option<&Path>,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let selected = {
        let selection = &app.active_tab().selected_range;
        !selection.is_empty()
            && selection.start < source_range.end
            && selection.end > source_range.start
    };
    #[cfg(test)]
    {
        VISUAL_HTML_IMAGE_ATOM_BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    let content = match app.preview_image_entry(&image.url, document_dir) {
        PreviewImageEntry::Ready(ready) => {
            // Same presentation rules as `preview_image_view`: supersampled
            // (SVG) entries present at their intrinsic display width, plain
            // rasters keep implicit pixel sizing, and authored HTML width/height
            // hints override both when present.
            let supersampled = ready.display_width != ready.width;
            let sized = resolve_html_img_display_size(
                image.width,
                image.height,
                ready.display_width as f32,
                ready.display_height as f32,
            );
            let rendered = img(ImageSource::Render(ready.image)).max_w_full();
            if let Some((width, height)) = sized {
                rendered.w(px(width)).h(px(height)).into_any_element()
            } else if supersampled {
                rendered
                    .w(px(ready.display_width as f32))
                    .into_any_element()
            } else {
                rendered.into_any_element()
            }
        }
        PreviewImageEntry::Pending | PreviewImageEntry::Error(_) => {
            let label = if image.alt.is_empty() {
                image.url.as_str()
            } else {
                image.alt.as_str()
            };
            div()
                .max_w(px(240.))
                .overflow_x_hidden()
                .truncate()
                .px_2()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(rgb(0xcbd5e1))
                .bg(rgb(0xf8fafc))
                .text_size(px(11.))
                .text_color(rgb(0x64748b))
                .child(label.to_string())
                .into_any_element()
        }
    };
    div()
        .relative()
        .flex_none()
        .max_w_full()
        .when(selected, |atom| atom.bg(rgba(PREVIEW_SELECTION_COLOR)))
        .child(content)
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
    #[cfg(test)] test_projection: Option<(String, Vec<Range<usize>>)>,
    #[cfg(test)] test_projection_styles: Option<Vec<InlineStyle>>,
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
        whitespace_caret: None,
        #[cfg(test)]
        test_projection,
        #[cfg(test)]
        test_projection_styles,
    }
    .into_any_element()
}

/// Source-backed mixed layout for Visual Edit. A focused formula is already a
/// source piece in `build_visual_projection`; other formulas remain image
/// atoms while adjacent prose keeps exact visible-to-source segments.
/// Inline raw-HTML `<img>` tags render through the same mixed layout as
/// image atoms, and focused tags reveal their authored source.
/// When the block has link/footnote navigation targets, the same flex-wrap
/// layout inserts a compact clickable icon after each navigable construct.
fn mixed_prose_current_line(lines: &mut Vec<Vec<gpui::AnyElement>>) -> &mut Vec<gpui::AnyElement> {
    if lines.is_empty() {
        lines.push(Vec::new());
    }
    lines
        .last_mut()
        .expect("mixed prose keeps at least one line")
}

fn mixed_prose_break_line(lines: &mut Vec<Vec<gpui::AnyElement>>) {
    lines.push(Vec::new());
}

fn mixed_prose_push(lines: &mut Vec<Vec<gpui::AnyElement>>, child: gpui::AnyElement) {
    mixed_prose_current_line(lines).push(child);
}

pub(super) fn visual_text_with_math_element(
    block: &VisualBlock,
    block_index: usize,
    app: &MarkionApp,
    display_scale: f32,
    document_dir: Option<&Path>,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let typography = app.typography_metrics();
    let inline_metrics = visual_inline_text_metrics(block, typography);
    let has_math = block.editable_runs.iter().any(|run| run.math.is_some());
    let has_html_image = block
        .editable_runs
        .iter()
        .any(|run| run.html_image.is_some());
    let nav_icons = visual_navigation_icons(block);
    if !has_math && !has_html_image && nav_icons.is_empty() {
        return visual_text_element(block, block_index, app, cx);
    }

    let projection = build_visual_projection_with_marked_range(
        app.active_tab().document.text(),
        block,
        app.active_tab().selected_range.clone(),
        app.active_tab().cursor_offset(),
        app.active_tab().marked_range.clone(),
    );
    #[cfg(test)]
    let focused_test_projection = visual_block_is_focused(app, block).then(|| {
        (
            projection.text.clone(),
            projection.revealed_source_ranges.clone(),
        )
    });
    #[cfg(test)]
    let focused_test_projection_styles = visual_block_is_focused(app, block).then(|| {
        projection
            .spans
            .iter()
            .filter(|span| !span.source)
            .map(|span| span.style)
            .collect::<Vec<_>>()
    });
    #[cfg(test)]
    let mut recorded_full_projection = false;
    let mut lines: Vec<Vec<gpui::AnyElement>> = vec![Vec::new()];
    let mut fragment_index = 0usize;
    let mut remaining_icons = nav_icons;
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
            mixed_prose_push(
                &mut lines,
                visual_math_atom(
                    app,
                    image,
                    math.source_range.clone(),
                    Some(inline_metrics),
                    cx,
                ),
            );
            fragment_index += 1;
            fragment_index += emit_navigation_icons_after(
                mixed_prose_current_line(&mut lines),
                &mut remaining_icons,
                segment.source_range.end,
                block_index,
                fragment_index,
                cx,
            );
            continue;
        }

        let html_image = (!projected_span.source).then(|| {
            block
                .editable_runs
                .iter()
                .find(|run| run.html_image.is_some() && run.content_range == segment.source_range)
        });
        if let Some(Some(run)) = html_image
            && let Some(image) = &run.html_image
        {
            mixed_prose_push(
                &mut lines,
                visual_html_image_atom(app, image, segment.source_range.clone(), document_dir, cx),
            );
            fragment_index += 1;
            fragment_index += emit_navigation_icons_after(
                mixed_prose_current_line(&mut lines),
                &mut remaining_icons,
                segment.source_range.end,
                block_index,
                fragment_index,
                cx,
            );
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
        let mut local_start = 0usize;
        for piece in visible.split_inclusive('\n') {
            if piece.is_empty() {
                continue;
            }
            let had_break = piece.ends_with('\n');
            let content = if had_break {
                let without_lf = piece.strip_suffix('\n').unwrap_or(piece);
                without_lf.strip_suffix('\r').unwrap_or(without_lf)
            } else {
                piece
            };
            if !content.is_empty() {
                if can_split {
                    let mut content_local = 0usize;
                    for fragment in content.split_inclusive(char::is_whitespace) {
                        if fragment.is_empty() {
                            continue;
                        }
                        let source_start = segment.source_range.start + local_start + content_local;
                        let source_range = source_start..source_start + fragment.len();
                        #[cfg(test)]
                        let (test_projection, test_projection_styles) = if !recorded_full_projection
                        {
                            recorded_full_projection = true;
                            (
                                focused_test_projection.clone(),
                                focused_test_projection_styles.clone(),
                            )
                        } else {
                            (None, None)
                        };
                        mixed_prose_push(
                            &mut lines,
                            visual_projection_fragment(
                                block_index,
                                fragment_index,
                                fragment.to_string(),
                                source_range.clone(),
                                style.clone(),
                                app,
                                cx,
                                #[cfg(test)]
                                test_projection,
                                #[cfg(test)]
                                test_projection_styles,
                            ),
                        );
                        fragment_index += 1;
                        content_local += fragment.len();
                        fragment_index += emit_navigation_icons_after(
                            mixed_prose_current_line(&mut lines),
                            &mut remaining_icons,
                            source_range.end,
                            block_index,
                            fragment_index,
                            cx,
                        );
                    }
                } else {
                    // Non-identity segment: display byte counts cannot index
                    // into the source range, so clamp the fragment to the
                    // segment's source bounds (rendered atoms resolve to their
                    // edges through `boundary_candidates`).
                    let source_start =
                        (segment.source_range.start + local_start).min(segment.source_range.end);
                    let source_end = source_start
                        .saturating_add(content.len())
                        .min(segment.source_range.end);
                    let source_range = source_start..source_end;
                    #[cfg(test)]
                    let (test_projection, test_projection_styles) = if !recorded_full_projection {
                        recorded_full_projection = true;
                        (
                            focused_test_projection.clone(),
                            focused_test_projection_styles.clone(),
                        )
                    } else {
                        (None, None)
                    };
                    mixed_prose_push(
                        &mut lines,
                        visual_projection_fragment(
                            block_index,
                            fragment_index,
                            content.to_string(),
                            source_range.clone(),
                            style.clone(),
                            app,
                            cx,
                            #[cfg(test)]
                            test_projection,
                            #[cfg(test)]
                            test_projection_styles,
                        ),
                    );
                    fragment_index += 1;
                    fragment_index += emit_navigation_icons_after(
                        mixed_prose_current_line(&mut lines),
                        &mut remaining_icons,
                        source_range.end,
                        block_index,
                        fragment_index,
                        cx,
                    );
                }
            }
            local_start += piece.len();
            if had_break {
                mixed_prose_break_line(&mut lines);
            }
        }
        fragment_index += emit_navigation_icons_after(
            mixed_prose_current_line(&mut lines),
            &mut remaining_icons,
            segment.source_range.end,
            block_index,
            fragment_index,
            cx,
        );
    }

    while lines.len() > 1 && lines.last().is_some_and(Vec::is_empty) {
        lines.pop();
    }

    div()
        .w_full()
        .flex()
        .flex_col()
        .children(lines.into_iter().enumerate().map(|(line_index, line)| {
            div()
                .w_full()
                .flex()
                .flex_wrap()
                .items_end()
                .debug_selector(move || format!("visual-mixed-line-{block_index}-{line_index}"))
                .children(line)
        }))
        .into_any_element()
}

fn visual_navigation_icons(block: &VisualBlock) -> Vec<(usize, VisualNavigationTarget)> {
    let mut icons = Vec::new();
    let mut index = 0usize;
    while index < block.editable_runs.len() {
        let Some(nav) = block.editable_runs[index].navigation.clone() else {
            index += 1;
            continue;
        };
        if matches!(&nav, VisualNavigationTarget::Url(url) if url.trim().is_empty()) {
            index += 1;
            continue;
        }
        let mut last = index;
        while last + 1 < block.editable_runs.len()
            && block.editable_runs[last + 1].navigation.as_ref() == Some(&nav)
        {
            last += 1;
        }
        icons.push((block.editable_runs[last].content_range.end, nav));
        index = last + 1;
    }
    icons
}

fn emit_navigation_icons_after(
    children: &mut Vec<gpui::AnyElement>,
    remaining: &mut Vec<(usize, VisualNavigationTarget)>,
    source_end: usize,
    block_index: usize,
    fragment_index: usize,
    cx: &mut Context<MarkionApp>,
) -> usize {
    let mut emitted = 0usize;
    while let Some(pos) = remaining.iter().position(|(after, _)| *after == source_end) {
        let (_, target) = remaining.remove(pos);
        children.push(visual_navigation_icon(
            block_index,
            fragment_index + emitted,
            target,
            cx,
        ));
        emitted += 1;
    }
    emitted
}

fn visual_navigation_icon(
    block_index: usize,
    fragment_index: usize,
    target: VisualNavigationTarget,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let glyph = match &target {
        VisualNavigationTarget::Url(_) => "↗",
        VisualNavigationTarget::Footnote { .. } => "↓",
    };
    let element_id = ElementId::from(("visual-nav-icon", block_index * 10_000 + fragment_index));
    div()
        .id(element_id)
        .ml(px(2.))
        .mr(px(1.))
        .px(px(3.))
        .rounded_sm()
        .text_size(px(11.))
        .line_height(px(14.))
        .text_color(rgb(PREVIEW_LINK_COLOR))
        .cursor(CursorStyle::PointingHand)
        .hover(|style| style.bg(rgba(0x2563eb22)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseDownEvent, window, cx| {
                // Keep the surrounding prose hit-target from placing a caret.
                cx.stop_propagation();
                let focus_handle = app.focus_handle.clone();
                window.focus(&focus_handle);
                app.activate_visual_navigation(&target, cx);
            }),
        )
        .child(glyph)
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
        .px_2()
        .py_1()
        .border_l_2()
        .border_color(rgb(0xcbd5e1))
        .bg(rgb(0xf8fafc))
        .font(code_slot_font(&app.resolved_font_families.code))
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
            whitespace_caret: None,
            #[cfg(test)]
            test_projection: None,
            #[cfg(test)]
            test_projection_styles: None,
        })
}

/// Hit-testable empty-paragraph surface for a `Whitespace` row. The caret
/// Y is derived from how many covered newlines sit before the source caret,
/// so repeated Enter at the document tail moves the insertion line down
/// the row. Clicks map the same way even when the row does not own the
/// caret. IME bounds still fall back to `visual_input_bounds` before the
/// row has painted.
pub(super) fn visual_whitespace_caret_element(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    line_height: f32,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let text = app.active_tab().document.text();
    let cursor = app.active_tab().cursor_offset();
    let source_range = block.source_range.clone();
    let caret_line = whitespace_caret_line(source_range.clone(), cursor, text);
    let anchor = source_range.start;
    let projection = VisualProjection {
        text: String::new(),
        segments: vec![markion::VisualProjectionSegment {
            display_range: 0..0,
            source_range: anchor..anchor,
        }],
        spans: Vec::new(),
        revealed_source_ranges: Vec::new(),
        source_anchor: anchor,
    };
    VisualEditableText {
        element_id: ElementId::from(("visual-whitespace-caret", block_index)),
        block_index,
        source_island: false,
        text: StyledText::new(SharedString::from("")),
        projection,
        source_selection: app.active_tab().selected_range.clone(),
        source_cursor: cursor,
        marked_range: app.active_tab().marked_range.clone(),
        caret_active: visual_block_owns_caret(app, block_index),
        navigation_active: false,
        entity: cx.entity(),
        whitespace_caret: Some(WhitespaceCaretLayout {
            caret_shift: px(whitespace_caret_y(caret_line, line_height)),
            source_range,
            line_height,
        }),
        #[cfg(test)]
        test_projection: None,
        #[cfg(test)]
        test_projection_styles: None,
    }
    .into_any_element()
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

/// Canonical GFM alert label, rendered bold in the callout title row. Like
/// list bullet glyphs this is document content, not app chrome, so it stays
/// untranslated.
fn callout_label(kind: AlertKind) -> &'static str {
    match kind {
        AlertKind::Note => "Note",
        AlertKind::Tip => "Tip",
        AlertKind::Important => "Important",
        AlertKind::Warning => "Warning",
        AlertKind::Caution => "Caution",
    }
}

/// GitHub-style alert accents; readable on both light and dark themes, like
/// the fixed quote-decoration grays used alongside them.
fn callout_accent_color(kind: AlertKind) -> Rgba {
    match kind {
        AlertKind::Note => rgb(0x0969da),
        AlertKind::Tip => rgb(0x1a7f37),
        AlertKind::Important => rgb(0x8250df),
        AlertKind::Warning => rgb(0x9a6700),
        AlertKind::Caution => rgb(0xcf222e),
    }
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

/// Sanity bound for pathological documents; ~49k px of tail whitespace is far
/// beyond any real document while keeping row heights bounded.
pub(super) const WHITESPACE_ROW_MAX_LINES: usize = 4096;

/// Covered-newline count used for virtual-list splice identity. Pixel height
/// is `count * paragraph_line_height` at render time.
pub(super) fn whitespace_clamped_line_count(line_count: usize) -> usize {
    line_count.clamp(1, WHITESPACE_ROW_MAX_LINES)
}

/// Height of a Visual Edit whitespace row: one body paragraph line per
/// covered newline, floored at one line. Uncapped up to the sanity bound so
/// trailing blank lines stay visible no matter how many the source carries.
pub(super) fn whitespace_row_height(line_count: usize, line_height: f32) -> f32 {
    whitespace_clamped_line_count(line_count) as f32 * line_height
}

pub(super) fn whitespace_painted_line_count(source_range: Range<usize>, text: &str) -> usize {
    let end = source_range.end.min(text.len());
    let start = source_range.start.min(end);
    whitespace_clamped_line_count(
        text[start..end]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count(),
    )
}

fn whitespace_newline_offsets(source_range: Range<usize>, text: &str) -> Vec<usize> {
    let end = source_range.end.min(text.len());
    let start = source_range.start.min(end);
    text[start..end]
        .bytes()
        .enumerate()
        .filter(|(_, byte)| *byte == b'\n')
        .map(|(index, _)| start + index)
        .collect()
}

/// Line index within a whitespace row for a source caret. Each covered
/// newline is one painted empty-paragraph line: the caret on the first
/// newline sits on line 0; each later newline in the range moves it down.
pub(super) fn whitespace_caret_line(
    source_range: Range<usize>,
    cursor: usize,
    text: &str,
) -> usize {
    let end = source_range.end.min(text.len());
    let start = source_range.start.min(end);
    let cursor = cursor.clamp(start, end);
    let newlines_before = text[start..cursor]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let max_line = whitespace_clamped_line_count(
        text[start..end]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count(),
    )
    .saturating_sub(1);
    newlines_before.min(max_line)
}

pub(super) fn whitespace_caret_y(line: usize, line_height: f32) -> f32 {
    let max_line = WHITESPACE_ROW_MAX_LINES.saturating_sub(1);
    line.min(max_line) as f32 * line_height
}

/// Source offset of painted line `line` inside a whitespace range. Line 0 is
/// the first covered newline byte; later lines walk subsequent newlines. The
/// result stays inside `[start, end)` so typing does not glue onto the next
/// block's first content byte.
pub(super) fn whitespace_source_at_line(
    source_range: Range<usize>,
    line: usize,
    text: &str,
) -> usize {
    let offsets = whitespace_newline_offsets(source_range.clone(), text);
    if offsets.is_empty() {
        return source_range.start.min(text.len());
    }
    offsets
        .get(line)
        .copied()
        .unwrap_or(*offsets.last().expect("non-empty newline list"))
}

/// Near-side landing offset when Up/Down enters a whitespace row: the first
/// covered newline when arriving from above, the last when arriving from below.
pub(super) fn whitespace_navigation_offset(
    source_range: Range<usize>,
    text: &str,
    from_above: bool,
) -> usize {
    let offsets = whitespace_newline_offsets(source_range.clone(), text);
    if offsets.is_empty() {
        return source_range.start.min(text.len());
    }
    if from_above {
        offsets[0]
    } else {
        *offsets.last().expect("non-empty newline list")
    }
}

pub(super) fn whitespace_source_at_y(
    source_range: Range<usize>,
    rel_y: Pixels,
    text: &str,
    line_height: f32,
) -> usize {
    let line = (f32::from(rel_y).max(0.) / line_height.max(1.)).floor() as usize;
    whitespace_source_at_line(source_range, line, text)
}

/// Viewport inset for the Visual Edit caret geometry gate. One default preview
/// line height so a caret on the last visible pixel has room for the next
/// glyph without pulling mid-pane clicks. Call sites pass the live
/// `preview_row_line_height` so custom typography stays aligned.
// Consumed only by the caret-viewport tests; production call sites pass the
// live typography metrics instead of this default.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) const VISUAL_CARET_VIEWPORT_INSET: f32 = PREVIEW_LINE_HEIGHT;

/// How the Visual Edit list should move to keep a caret usable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum VisualCaretScrollAction {
    None,
    /// Target row has no measured height and sits below the measured window.
    PinItem,
    /// Item is known but has no usable bounds (typically above the scroll top).
    RevealItem,
    /// Scroll the current logical top by this pixel delta (positive = down).
    Pixel(Pixels),
}

/// Pure geometry gate: no document or derived-cache access.
pub(super) fn visual_caret_scroll_action(
    viewport: Bounds<Pixels>,
    caret: Option<Bounds<Pixels>>,
    item_bounds: Option<Bounds<Pixels>>,
    inset: Pixels,
    unmeasured_below: bool,
) -> VisualCaretScrollAction {
    let target = caret
        .filter(|bounds| bounds.size.height > px(0.))
        .or(item_bounds.filter(|bounds| bounds.size.height > px(0.)));
    if let Some(rect) = target {
        return visual_caret_pixel_delta(viewport, rect, inset)
            .map(VisualCaretScrollAction::Pixel)
            .unwrap_or(VisualCaretScrollAction::None);
    }
    if unmeasured_below {
        return VisualCaretScrollAction::PinItem;
    }
    if viewport.size.height <= px(0.) {
        VisualCaretScrollAction::None
    } else {
        VisualCaretScrollAction::RevealItem
    }
}

/// Minimal pixel delta that places `target` inside `viewport` inset by `inset`.
/// `None` when the target is already inside, or when either rect has no height.
pub(super) fn visual_caret_pixel_delta(
    viewport: Bounds<Pixels>,
    target: Bounds<Pixels>,
    inset: Pixels,
) -> Option<Pixels> {
    if viewport.size.height <= px(0.) || target.size.height <= px(0.) {
        return None;
    }
    let mut view_top = viewport.top() + inset;
    let mut view_bottom = viewport.bottom() - inset;
    if view_bottom <= view_top {
        view_top = viewport.top();
        view_bottom = viewport.bottom();
    }
    if target.bottom() > view_bottom {
        Some(target.bottom() - view_bottom)
    } else if target.top() < view_top {
        Some(target.top() - view_top)
    } else {
        None
    }
}

/// Apply the geometry gate to a pending Visual Edit caret reveal.
/// Returns true when a scroll was requested.
pub(super) fn apply_visual_caret_reveal(
    list: &ListState,
    item_ix: usize,
    caret: Option<Bounds<Pixels>>,
    inset: Pixels,
) -> bool {
    let viewport = list.viewport_bounds();
    let item_bounds = list.bounds_for_item(item_ix);
    let top = list.logical_scroll_top();
    let unmeasured_below = item_bounds.is_none() && item_ix > top.item_ix;
    match visual_caret_scroll_action(viewport, caret, item_bounds, inset, unmeasured_below) {
        VisualCaretScrollAction::None => false,
        VisualCaretScrollAction::PinItem => {
            list.scroll_to(gpui::ListOffset {
                item_ix,
                offset_in_item: px(0.),
            });
            true
        }
        VisualCaretScrollAction::RevealItem => {
            list.scroll_to_reveal_item(item_ix);
            true
        }
        VisualCaretScrollAction::Pixel(delta) => {
            if delta == px(0.) {
                return false;
            }
            list.scroll_to(gpui::ListOffset {
                item_ix: top.item_ix,
                offset_in_item: top.offset_in_item + delta,
            });
            true
        }
    }
}

/// Scroll the Visual Edit list just enough to keep `caret` inside the
/// viewport inset. Returns true when a scroll was requested.
pub(super) fn follow_visual_caret_in_list(
    list: &ListState,
    caret: Bounds<Pixels>,
    inset: Pixels,
) -> bool {
    let Some(delta) = visual_caret_pixel_delta(list.viewport_bounds(), caret, inset) else {
        return false;
    };
    if delta == px(0.) {
        return false;
    }
    let top = list.logical_scroll_top();
    list.scroll_to(gpui::ListOffset {
        item_ix: top.item_ix,
        offset_in_item: top.offset_in_item + delta,
    });
    true
}

/// Trailing Visual Edit list item after the last `VisualBlock`. Empty
/// documents keep the placeholder surface and have no spacer.
pub(super) fn visual_list_item_count(block_count: usize) -> usize {
    if block_count == 0 { 0 } else { block_count + 1 }
}

/// Half the current Visual Edit viewport; 0 before the first layout.
pub(super) fn visual_end_padding_height(viewport_height: Pixels) -> Pixels {
    if viewport_height <= px(0.) {
        px(0.)
    } else {
        px(f32::from(viewport_height) * 0.5)
    }
}

pub(super) fn ensure_visual_list_spacer(list: &ListState, block_count: usize) {
    let desired = visual_list_item_count(block_count);
    let current = list.item_count();
    if current == desired {
        return;
    }
    if current < desired {
        list.splice(current..current, desired - current);
    } else {
        list.splice(desired..current, 0);
    }
}

pub(super) fn visual_end_padding_view(
    app: &MarkionApp,
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    let height = app
        .active_tab()
        .visual_end_padding_height
        .unwrap_or_else(|| {
            visual_end_padding_height(app.active_tab().visual_list.viewport_bounds().size.height)
        });
    div()
        .id("visual-document-end-padding")
        .debug_selector(|| "visual-document-end-padding".to_string())
        .w_full()
        .h(height)
        .cursor(CursorStyle::IBeam)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|app, event: &MouseDownEvent, window, cx| {
                let focus_handle = app.focus_handle.clone();
                window.focus(&focus_handle);
                app.file_tree_query_focused = false;
                app.search_focus = None;
                app.search_control_focus = None;
                app.input_marked_len = 0;
                app.active_tab_mut().clear_preview_selection();
                let end = app.active_tab().document.text().len();
                if event.modifiers.shift {
                    app.select_to(end, cx);
                } else {
                    app.move_to(end, cx);
                }
            }),
        )
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
    let is_whitespace = matches!(block.kind, VisualBlockKind::Whitespace);
    let is_reference_definition = matches!(block.kind, VisualBlockKind::ReferenceDefinition);
    // A callout title row owns only structural marker bytes; focused, it
    // reveals them through the inline projection instead of a source-island
    // box, so it is excluded from the conservative gate like whitespace rows.
    let is_callout_title = matches!(block.kind, VisualBlockKind::CalloutTitle { .. });
    let has_html_image = block
        .editable_runs
        .iter()
        .any(|run| run.html_image.is_some());
    let always_source = matches!(
        block.source_island,
        Some(
            VisualSourceIslandKind::FrontMatter
                | VisualSourceIslandKind::Code
                | VisualSourceIslandKind::Unsupported
        )
    );
    // Conservative inline-HTML fragments stay in the mixed rendered path
    // (verbatim tag atoms) instead of promoting the whole paragraph to a
    // source island. Front matter, unclosed code, and unsupported parser
    // gaps still use the whole-block source box.
    // A Whitespace row that owns the caret is ordinary inter-paragraph
    // spacing, not a code-like block. Promoting it to a source-island box
    // (border + padding + monospace + gray background) makes a normal blank
    // line look like a source island — see change
    // `fix-visual-edit-whitespace-caret-box`. Whitespace owning the caret
    // is painted as a thin caret line in the `Whitespace` arm below.
    // Link reference definitions keep a muted editable row without island
    // chrome even when they own the caret (empty editable_runs by design).
    // Empty editable_runs alone must not island a rendered kind: empty ATX
    // headings and empty list items keep heading/list typography and reveal
    // their structural prefix. Island chrome is reserved for blocks that
    // actually carry a source_island kind (front matter, unclosed code,
    // residual unsupported gaps).
    let focused_conservative = owns_caret
        && !is_whitespace
        && !is_reference_definition
        && !is_callout_title
        && block.editor.is_none()
        && !has_html_image
        && block.source_island.is_some();
    if focused_conservative || always_source {
        let row = visual_source_island_view(app, block, block_index, cx);
        let blocks = app.active_tab().document.visual_blocks_shared();
        return if block_can_transform_at(&blocks, block_index) {
            visual_block_chrome(app, block, block_index, owns_caret, row, cx)
        } else {
            row
        };
    }

    let row = match &block.kind {
        VisualBlockKind::Heading { level } => {
            let heading_size = typography.heading_font_size((*level).into());
            let size = px(heading_size);
            div()
                .mt_2()
                .mb_2()
                .text_size(size)
                .line_height(px(default_text_line_height(heading_size)))
                .font_weight(FontWeight::BOLD)
                .child(visual_text_with_math_element(
                    block,
                    block_index,
                    app,
                    display_scale,
                    document_dir,
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
                document_dir,
                cx,
            )),
        VisualBlockKind::ListItem {
            level,
            ordered,
            index,
            checked,
        } => {
            let cursor = app.active_tab().cursor_offset();
            let prefix_revealed = block.block_prefix.as_ref().is_some_and(|prefix| {
                prefix.source_range.contains(&cursor)
                    || cursor == prefix.source_range.end
                    || (owns_caret && block.editable_runs.is_empty())
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
                            document_dir,
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
                document_dir,
                cx,
            )),
        VisualBlockKind::Image {
            alt: image_alt,
            url,
            title: image_title,
            ..
        } => {
            let offset = block.source_range.start;
            let exact_image = inline_image_at(app.active_tab().document.text(), offset);
            let presentation = exact_image
                .as_ref()
                .and_then(|image| image.presentation)
                .unwrap_or_default();
            let caption = exact_image
                .as_ref()
                .and_then(|image| image.title.as_deref())
                .or(image_title.as_deref())
                .filter(|title| !title.is_empty())
                .or_else(|| (!image_alt.is_empty()).then_some(image_alt.as_str()));
            let image = div()
                .w(gpui::relative(presentation.width_percent as f32 / 100.))
                .child(preview_image_view(app, url, document_dir, None, None));
            let image = match presentation.alignment {
                ImageAlignment::Left => div().w_full().flex().items_start().child(image),
                ImageAlignment::Center => div().w_full().flex().items_center().child(image),
                ImageAlignment::Right => div().w_full().flex().items_end().child(image),
            };
            div()
                .mb_3()
                .cursor(CursorStyle::PointingHand)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |app, _, _, cx| app.move_to(offset, cx)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .child(image)
                        .children(caption.map(|text| {
                            div()
                                .mt_1()
                                .text_size(px(11.))
                                .text_color(rgb(0x64748b))
                                .child(text.to_string())
                        }))
                        .children((owns_caret && exact_image.is_some()).then(|| {
                            visual_image_controls(offset, presentation, app.language, cx)
                        })),
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
        VisualBlockKind::Table { rows, .. } => {
            div().child(visual_table_view(app, block, block_index, rows, cx))
        }
        VisualBlockKind::CalloutTitle { kind } => {
            if owns_caret {
                // Focused, the projection reveals the authored marker line
                // (`> [!NOTE]`) as an editable source-backed range.
                div()
                    .line_height(px(typography.paragraph_line_height))
                    .text_size(px(typography.rendered_font_size))
                    .child(visual_text_element(block, block_index, app, cx))
            } else {
                // Clicking the label lands the caret just inside the marker
                // line's end, mirroring keyboard entry into the row.
                let source_range = &block.source_range;
                let line_end = app.active_tab().document.text()[source_range.clone()]
                    .find('\n')
                    .map_or(source_range.end, |relative| source_range.start + relative);
                let click_target = if line_end > source_range.start {
                    line_end - 1
                } else {
                    source_range.start
                };
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(typography.rendered_font_size))
                    .text_color(callout_accent_color(*kind))
                    .cursor(CursorStyle::IBeam)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |app, _, _, cx| app.move_to(click_target, cx)),
                    )
                    .child(callout_label(*kind))
            }
        }
        VisualBlockKind::Whitespace => {
            let text = app.active_tab().document.text();
            let source_range = block.source_range.clone();
            let line_count = text[source_range.clone()]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count();
            let line_height = typography.paragraph_line_height;
            let row_height = whitespace_row_height(line_count, line_height);

            // First-class empty line: body paragraph height, I-beam, and
            // click mapping whether or not the row owns the caret. The
            // caret itself is painted only when `owns_caret`.
            div()
                .h(px(row_height))
                .cursor(CursorStyle::IBeam)
                .debug_selector(|| "visual-whitespace-gap".to_string())
                .child(visual_whitespace_caret_element(
                    app,
                    block,
                    block_index,
                    line_height,
                    cx,
                ))
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
                                        None,
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
                // Diagram fences (e.g. `mermaid`) layer a rendered image on
                // top of the same source-backed payload editor used by ordinary
                // code blocks. Non-diagram CodeBlocks keep the highlighted
                // code editor.
                if diagram_backend_id(language.as_deref()).is_some() {
                    visual_diagram_editor(app, block, block_index, language.as_deref(), payload, cx)
                } else {
                    visual_code_editor(app, block.id, block_index, language.as_deref(), payload, cx)
                }
            } else {
                visual_source_island_view(app, block, block_index, cx)
            }
        }
        VisualBlockKind::Unsupported => visual_source_island_view(app, block, block_index, cx),
        VisualBlockKind::Html { html } => {
            if let Some(VisualBlockEditor::Html { payload }) = block.editor.as_ref() {
                visual_html_editor(app, block, block_index, html, payload, document_dir, cx)
            } else {
                html_preview_block_view(app, html, block_index, document_dir, cx)
            }
        }
        VisualBlockKind::FootnoteDefinition { label } => div()
            .mb(px(typography.paragraph_spacing))
            .mt_2()
            .pt_2()
            .border_t_1()
            .border_color(rgb(0xe2e8f0))
            .flex()
            .items_start()
            .gap_2()
            .text_size(px(typography.rendered_font_size))
            .line_height(px(typography.paragraph_line_height))
            .child(
                div()
                    .flex_none()
                    .text_size(px(typography.rendered_font_size * 0.75))
                    .text_color(rgb(0x64748b))
                    .child(format!("[{label}]")),
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
                        document_dir,
                        cx,
                    )),
            ),
        VisualBlockKind::ReferenceDefinition => {
            visual_reference_definition_view(app, block, block_index, cx)
        }
    };
    let blocks = app.active_tab().document.visual_blocks_shared();
    let row = if block_can_transform_at(&blocks, block_index) {
        visual_block_chrome(app, block, block_index, owns_caret, row, cx)
    } else {
        row
    };
    if let Some(quote) = &block.quote_context {
        let (padding_top, padding_bottom) = match quote.edge {
            VisualQuoteGroupEdge::Only => (4., 4.),
            VisualQuoteGroupEdge::First => (4., 0.),
            VisualQuoteGroupEdge::Middle => (0., 0.),
            VisualQuoteGroupEdge::Last => (0., 4.),
        };
        div()
            .ml(px(quote.depth.saturating_sub(1) as f32 * 8.))
            .pl_3()
            .pt(px(padding_top))
            .pb(px(padding_bottom))
            .border_l_1()
            .border_color(rgb(0x94a3b8))
            .text_color(rgb(0x475569))
            .child(row)
    } else {
        row
    }
}

fn visual_block_chrome(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    owns_caret: bool,
    content: Div,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let palette = app.palette();
    let target = BlockTarget::from_block(app.active_tab().document.version(), block);
    let blocks = app.active_tab().document.visual_blocks_shared();
    let reorderable = block_can_reorder_at(&blocks, block_index);
    let before_target = target.clone();
    let after_target = target.clone();
    let drag_target = target.clone();
    let menu_target = target.clone();
    let hover_group = format!("visual-block-row-hover-{}", block.id.as_u64());

    div()
        .relative()
        .group(hover_group.clone())
        .debug_selector(move || format!("visual-block-row-{block_index}"))
        .w_full()
        .on_mouse_up(
            MouseButton::Right,
            cx.listener(move |app, event: &MouseUpEvent, window, cx| {
                cx.stop_propagation();
                window.focus(&app.focus_handle(cx));
                app.open_visual_block_menu(menu_target.clone(), event.position, cx);
            }),
        )
        .child(
            div()
                .debug_selector(move || format!("visual-block-content-{block_index}"))
                .w_full()
                .min_w_0()
                .child(content),
        )
        .when(reorderable, |row| {
            row.child(
                div()
                    .id(("visual-block-drag-grip", block.id.as_u64()))
                    .debug_selector(move || format!("visual-block-drag-grip-{block_index}"))
                    .absolute()
                    .left(px(-14.))
                    .top(px(0.))
                    .w(px(14.))
                    .h(px(22.))
                    .rounded_sm()
                    .text_size(px(12.))
                    .text_color(palette.muted)
                    .cursor(CursorStyle::OpenHand)
                    .opacity(if owns_caret { 1. } else { 0. })
                    .group_hover(hover_group, |style| style.opacity(1.))
                    .active(|style| style.opacity(1.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(move |style| style.bg(palette.surface_bg))
                    .child("⠿")
                    .on_drag(
                        DraggedVisualBlock {
                            target: drag_target,
                        },
                        move |_, _, _, cx| cx.new(|_| Empty),
                    ),
            )
            .child(
                div()
                    .id(("visual-block-drop-before", block.id.as_u64()))
                    .debug_selector(move || format!("visual-block-drop-before-{block_index}"))
                    .absolute()
                    .top(px(-3.))
                    .left_0()
                    .right_0()
                    .h(px(6.))
                    .on_drop::<DraggedVisualBlock>(cx.listener(
                        move |app, dragged: &DraggedVisualBlock, _, cx| {
                            app.reorder_visual_block(
                                dragged.target.clone(),
                                before_target.clone(),
                                BlockPlacement::Before,
                                cx,
                            );
                        },
                    )),
            )
            .child(
                div()
                    .id(("visual-block-drop-after", block.id.as_u64()))
                    .debug_selector(move || format!("visual-block-drop-after-{block_index}"))
                    .absolute()
                    .bottom(px(-3.))
                    .left_0()
                    .right_0()
                    .h(px(6.))
                    .on_drop::<DraggedVisualBlock>(cx.listener(
                        move |app, dragged: &DraggedVisualBlock, _, cx| {
                            app.reorder_visual_block(
                                dragged.target.clone(),
                                after_target.clone(),
                                BlockPlacement::After,
                                cx,
                            );
                        },
                    )),
            )
        })
}

pub(super) fn visual_block_menu(
    language: Language,
    state: BlockMenuState,
    presentation: BlockMenuPresentation,
    palette: ThemePalette,
    max_height: Pixels,
    cx: &mut Context<MarkionApp>,
) -> impl IntoElement {
    let submenu = state.submenu;
    let root_items = state.root_items();
    let mut root_panel = div()
        .id("visual-block-menu-root-panel")
        .debug_selector(|| "visual-block-menu-panel".to_string())
        .w(px(202.))
        .max_h(max_height)
        .overflow_y_scroll()
        .scrollbar_width(px(8.))
        .occlude()
        .p(px(4.))
        .bg(palette.panel_bg)
        .border_1()
        .border_color(palette.border)
        .rounded_lg()
        .shadow_lg()
        .flex()
        .flex_col();
    for (index, item) in root_items.iter().copied().enumerate() {
        root_panel = root_panel.child(block_menu_item_button(
            block_menu_item_element_id(item),
            block_menu_item_label(language, item),
            item,
            state.root_selected == index,
            block_menu_item_is_current(item, presentation),
            presentation.item_enabled(item),
            Some(BlockMenuPointerTarget::Root(index)),
            matches!(item, BlockMenuItem::Delete),
            palette,
            cx,
        ));
        if block_menu_item_is_followed_by_separator(item) {
            root_panel = root_panel.child(block_menu_separator(palette));
        }
    }
    div()
        .id(("visual-block-menu", state.target.block_id.as_u64()))
        .flex()
        .items_start()
        .gap_1()
        .child(root_panel)
        .when_some(submenu, |menu, submenu| {
            menu.child(visual_block_submenu(
                language,
                submenu,
                state.submenu_selected,
                presentation,
                palette,
                max_height,
                cx,
            ))
        })
}

fn block_menu_item_element_id(item: BlockMenuItem) -> &'static str {
    match item {
        BlockMenuItem::SelectionFormat(SelectionFormatAction::Bold) => {
            "visual-selection-format-bold"
        }
        BlockMenuItem::SelectionFormat(SelectionFormatAction::Italic) => {
            "visual-selection-format-italic"
        }
        BlockMenuItem::SelectionFormat(SelectionFormatAction::InlineCode) => {
            "visual-selection-format-inline-code"
        }
        BlockMenuItem::SelectionFormat(SelectionFormatAction::Link) => {
            "visual-selection-format-link"
        }
        BlockMenuItem::Submenu(BlockMenuSubmenu::TextAndHeadings) => "visual-block-text-headings",
        BlockMenuItem::Submenu(BlockMenuSubmenu::Lists) => "visual-block-lists",
        BlockMenuItem::Transform(transform) => {
            visual_block_transform_element_id(block_transform_index(transform))
        }
        BlockMenuItem::Duplicate => "visual-block-duplicate",
        BlockMenuItem::MoveUp => "visual-block-move-up",
        BlockMenuItem::MoveDown => "visual-block-move-down",
        BlockMenuItem::Delete => "visual-block-delete",
    }
}

fn block_menu_item_label(language: Language, item: BlockMenuItem) -> String {
    match item {
        BlockMenuItem::SelectionFormat(SelectionFormatAction::Bold) => {
            t(language, Msg::ItemBold).to_string()
        }
        BlockMenuItem::SelectionFormat(SelectionFormatAction::Italic) => {
            t(language, Msg::ItemItalic).to_string()
        }
        BlockMenuItem::SelectionFormat(SelectionFormatAction::InlineCode) => {
            t(language, Msg::ItemInlineCode).to_string()
        }
        BlockMenuItem::SelectionFormat(SelectionFormatAction::Link) => {
            t(language, Msg::ItemLink).to_string()
        }
        BlockMenuItem::Submenu(BlockMenuSubmenu::TextAndHeadings) => {
            p1_t(language, P1Msg::TextAndHeadings).to_string()
        }
        BlockMenuItem::Submenu(BlockMenuSubmenu::Lists) => p1_t(language, P1Msg::Lists).to_string(),
        BlockMenuItem::Transform(transform) => block_transform_label(language, transform),
        BlockMenuItem::Duplicate => p1_t(language, P1Msg::DuplicateBlock).to_string(),
        BlockMenuItem::MoveUp => p1_t(language, P1Msg::MoveUp).to_string(),
        BlockMenuItem::MoveDown => p1_t(language, P1Msg::MoveDown).to_string(),
        BlockMenuItem::Delete => p1_t(language, P1Msg::DeleteBlock).to_string(),
    }
}

fn block_menu_item_is_current(item: BlockMenuItem, presentation: BlockMenuPresentation) -> bool {
    match item {
        BlockMenuItem::Submenu(BlockMenuSubmenu::TextAndHeadings) => matches!(
            presentation.current,
            BlockTransform::Text | BlockTransform::Heading(_)
        ),
        BlockMenuItem::Submenu(BlockMenuSubmenu::Lists) => matches!(
            presentation.current,
            BlockTransform::BulletedList | BlockTransform::NumberedList | BlockTransform::TaskList
        ),
        BlockMenuItem::Transform(transform) => presentation.current == transform,
        BlockMenuItem::SelectionFormat(_)
        | BlockMenuItem::Duplicate
        | BlockMenuItem::MoveUp
        | BlockMenuItem::MoveDown
        | BlockMenuItem::Delete => false,
    }
}

fn block_menu_item_is_followed_by_separator(item: BlockMenuItem) -> bool {
    matches!(
        item,
        BlockMenuItem::SelectionFormat(SelectionFormatAction::Link)
            | BlockMenuItem::Transform(BlockTransform::Table)
            | BlockMenuItem::MoveDown
    )
}

#[derive(Clone, Copy)]
enum BlockMenuPointerTarget {
    Root(usize),
    Submenu(BlockMenuSubmenu, usize),
}

fn visual_block_submenu(
    language: Language,
    submenu: BlockMenuSubmenu,
    selected: usize,
    presentation: BlockMenuPresentation,
    palette: ThemePalette,
    max_height: Pixels,
    cx: &mut Context<MarkionApp>,
) -> impl IntoElement {
    let mut panel = div()
        .id("visual-block-menu-submenu-panel")
        .debug_selector(|| "visual-block-submenu-panel".to_string())
        .w(px(202.))
        .max_h(max_height)
        .overflow_y_scroll()
        .scrollbar_width(px(8.))
        .occlude()
        .p(px(4.))
        .bg(palette.panel_bg)
        .border_1()
        .border_color(palette.border)
        .rounded_lg()
        .shadow_lg()
        .flex()
        .flex_col();
    for (index, item) in submenu.items().iter().copied().enumerate() {
        let BlockMenuItem::Transform(transform) = item else {
            continue;
        };
        let transform_index = block_transform_index(transform);
        panel = panel.child(block_menu_item_button(
            visual_block_transform_element_id(transform_index),
            block_transform_label(language, transform),
            item,
            selected == index,
            presentation.current == transform,
            true,
            Some(BlockMenuPointerTarget::Submenu(submenu, index)),
            false,
            palette,
            cx,
        ));
    }
    panel
}

fn block_menu_separator(palette: ThemePalette) -> Div {
    div().my(px(3.)).h(px(1.)).bg(palette.border)
}

#[allow(clippy::too_many_arguments)]
fn block_menu_item_button(
    id: &'static str,
    label: String,
    item: BlockMenuItem,
    selected: bool,
    current: bool,
    enabled: bool,
    pointer_target: Option<BlockMenuPointerTarget>,
    destructive: bool,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> impl IntoElement {
    let trailing = match item {
        BlockMenuItem::Submenu(_) if current => "✓  ›",
        BlockMenuItem::Submenu(_) => "›",
        _ if current => "✓",
        _ => "",
    };
    div()
        .id(id)
        .debug_selector(move || {
            if enabled {
                id.to_string()
            } else {
                format!("{id}-disabled")
            }
        })
        .h(px(26.))
        .px_2()
        .rounded_sm()
        .text_size(px(12.))
        .text_color(if destructive && enabled {
            rgb(0xdc2626)
        } else {
            palette.text
        })
        .opacity(if enabled { 1. } else { 0.42 })
        .when(enabled, |row| {
            row.cursor_pointer()
                .hover(move |style| style.bg(palette.surface_bg))
        })
        .when(selected && enabled, |row| row.bg(palette.surface_bg))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .min_w_0()
                .when(current, |label| label.font_weight(FontWeight::SEMIBOLD))
                .child(label),
        )
        .child(
            div()
                .ml_2()
                .flex_none()
                .text_color(palette.muted)
                .when(current, |indicator| {
                    indicator.debug_selector(|| "visual-block-current-indicator".to_string())
                })
                .child(trailing),
        )
        .when(enabled, |row| {
            row.on_mouse_move(cx.listener(
                move |app, _: &MouseMoveEvent, _, cx| match pointer_target {
                    Some(BlockMenuPointerTarget::Root(index)) => {
                        app.select_visual_block_menu_root(
                            index,
                            matches!(item, BlockMenuItem::Submenu(_)),
                            cx,
                        );
                    }
                    Some(BlockMenuPointerTarget::Submenu(submenu, index)) => {
                        app.select_visual_block_menu_submenu(submenu, index, cx);
                    }
                    None => {}
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |app, _: &MouseUpEvent, _, cx| {
                    app.activate_visual_block_menu_item(item, cx);
                }),
            )
        })
}

fn visual_block_transform_element_id(index: usize) -> &'static str {
    const IDS: [&str; 14] = [
        "visual-block-transform-0",
        "visual-block-transform-1",
        "visual-block-transform-2",
        "visual-block-transform-3",
        "visual-block-transform-4",
        "visual-block-transform-5",
        "visual-block-transform-6",
        "visual-block-transform-7",
        "visual-block-transform-8",
        "visual-block-transform-9",
        "visual-block-transform-10",
        "visual-block-transform-11",
        "visual-block-transform-12",
        "visual-block-transform-13",
    ];
    IDS[index.min(IDS.len() - 1)]
}

fn block_transform_index(transform: BlockTransform) -> usize {
    match transform {
        BlockTransform::Text => 0,
        BlockTransform::Heading(level) => usize::from(level.clamp(1, 6)),
        BlockTransform::BulletedList => 7,
        BlockTransform::NumberedList => 8,
        BlockTransform::TaskList => 9,
        BlockTransform::Quote => 10,
        BlockTransform::CodeBlock => 11,
        BlockTransform::Divider => 12,
        BlockTransform::Table => 13,
    }
}

fn block_transform_label(language: Language, transform: BlockTransform) -> String {
    match transform {
        BlockTransform::Text => p1_t(language, P1Msg::TextBlock).to_string(),
        BlockTransform::Heading(level) => p1_tf(language, P1Msg::Heading, &[&level.to_string()]),
        BlockTransform::BulletedList => p1_t(language, P1Msg::BulletedList).to_string(),
        BlockTransform::NumberedList => p1_t(language, P1Msg::NumberedList).to_string(),
        BlockTransform::TaskList => p1_t(language, P1Msg::TaskList).to_string(),
        BlockTransform::Quote => p1_t(language, P1Msg::Quote).to_string(),
        BlockTransform::CodeBlock => p1_t(language, P1Msg::CodeBlock).to_string(),
        BlockTransform::Divider => p1_t(language, P1Msg::Divider).to_string(),
        BlockTransform::Table => p1_t(language, P1Msg::Table).to_string(),
    }
}

/// Editable link-reference definition row without Unsupported island chrome.
pub(super) fn visual_reference_definition_view(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    let source = app.active_tab().document.text()[block.source_range.clone()].to_string();
    let source_len = source.len();
    div()
        .mb_1()
        .text_color(rgb(0x64748b))
        .font(code_slot_font(&app.resolved_font_families.code))
        .text_size(px(typography.source_island_font_size))
        .line_height(px(typography.source_island_line_height))
        .child(VisualEditableText {
            element_id: ElementId::from(("visual-reference-definition", block_index)),
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
            whitespace_caret: None,
            #[cfg(test)]
            test_projection: None,
            #[cfg(test)]
            test_projection_styles: None,
        })
}

fn visual_editor_field_element(
    app: &MarkionApp,
    block_index: usize,
    field: &VisualEditorField,
    element_id: ElementId,
    styled_text: Option<StyledText>,
    cell_rich: Option<&RichText>,
    cx: &mut Context<MarkionApp>,
) -> gpui::AnyElement {
    let source = app.active_tab().document.text();
    let source_cursor = app.active_tab().cursor_offset();
    let block_owns_caret = visual_block_owns_caret(app, block_index);
    let caret_active = block_owns_caret
        && source_cursor >= field.source_range.start
        && source_cursor <= field.source_range.end;
    let marked_range = app.active_tab().marked_range.clone().filter(|marked| {
        marked.start >= field.source_range.start && marked.end <= field.source_range.end
    });

    // Table cells render inline formatting (bold, links, etc.) while unfocused,
    // and reveal the authored source markup when focused for editing -
    // mirroring how non-table visual blocks reveal inline constructs.
    let (projection, text, highlights) = if let Some(rich) = cell_rich {
        if caret_active {
            let proj = visual_editor_field_projection(source, field);
            let txt = proj.text.clone();
            let hl = Vec::new();
            (proj, txt, hl)
        } else {
            let (proj, hl) = table_cell_rendered_projection(rich, field);
            let txt = proj.text.clone();
            (proj, txt, hl)
        }
    } else {
        let proj = visual_editor_field_projection(source, field);
        let txt = proj.text.clone();
        (proj, txt, Vec::new())
    };

    #[cfg(test)]
    let test_projection = caret_active.then_some((text.clone(), Vec::new()));
    let styled = if !highlights.is_empty() {
        StyledText::new(SharedString::from(text.clone())).with_highlights(highlights)
    } else {
        styled_text.unwrap_or_else(|| StyledText::new(SharedString::from(text)))
    };
    VisualEditableText {
        element_id,
        block_index,
        source_island: false,
        text: styled,
        projection,
        source_selection: app.active_tab().selected_range.clone(),
        source_cursor,
        marked_range,
        caret_active,
        navigation_active: caret_active || !block_owns_caret,
        entity: cx.entity(),
        whitespace_caret: None,
        #[cfg(test)]
        test_projection,
        #[cfg(test)]
        test_projection_styles: None,
    }
    .into_any_element()
}

/// Builds a rendered (non-source-revealing) projection for a table cell from
/// its already-parsed `RichText`. The display text is the rendered content
/// (e.g. "bold" for `**bold**`); segments map the whole run to the cell's
/// source range so clicks resolve inside the cell; spans carry the inline
/// styles for highlight generation.
fn table_cell_rendered_projection(
    rich: &RichText,
    field: &VisualEditorField,
) -> (VisualProjection, Vec<(Range<usize>, HighlightStyle)>) {
    let text = rich.text.clone();
    let mut highlights = Vec::new();
    let mut spans = Vec::new();
    let mut offset = 0usize;
    for span in &rich.spans {
        let range = offset..offset + span.text.len();
        offset = range.end;
        if let Some(style) = visual_highlight_style(span.style, span.link.is_some()) {
            highlights.push((range.clone(), style));
        }
        spans.push(markion::VisualProjectionSpan {
            display_range: range,
            style: span.style,
            link: span.link.is_some(),
            source: false,
        });
    }
    let segments = if text.is_empty() {
        Vec::new()
    } else {
        vec![markion::VisualProjectionSegment {
            display_range: 0..text.len(),
            source_range: field.source_range.clone(),
        }]
    };
    let projection = VisualProjection {
        text,
        segments,
        spans,
        revealed_source_ranges: Vec::new(),
        source_anchor: field.source_range.start,
    };
    (projection, highlights)
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
        VisualEditorFieldKind::CodePayload
        | VisualEditorFieldKind::MathPayload
        | VisualEditorFieldKind::HtmlSource => None,
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
    let palette = code_palette(app.code_theme);
    let code = &app.active_tab().document.text()[payload.source_range.clone()];
    let highlighted = app.highlighted_code(language, code);
    let (styled, _) = code_block_text(&highlighted, palette);
    div()
        .mb_3()
        .p_3()
        .rounded_md()
        .bg(palette.bg)
        .text_color(palette.text)
        .font(code_slot_font(&app.resolved_font_families.code))
        .text_size(px(typography.code_font_size))
        .line_height(px(typography.code_line_height))
        .child(code_block_header(
            app,
            language,
            code.to_string(),
            palette,
            cx,
        ))
        .child(visual_editor_field_element(
            app,
            block_index,
            payload,
            ElementId::from(("visual-code-payload", block_id.as_u64())),
            Some(styled),
            None,
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
    let entry = app.math_entry(
        latex,
        MathLayoutStyle::Display,
        typography.display_math_font_size,
        1.0,
        display_scale,
        app.palette().text,
    );
    let forced = !matches!(entry, MathCacheEntry::Ready(_));
    let presentation = match entry {
        MathCacheEntry::Ready(image) => {
            div()
                .w_full()
                .py_2()
                .flex()
                .justify_center()
                .child(visual_math_atom(
                    app,
                    image,
                    block.source_range.clone(),
                    None,
                    cx,
                ))
        }
        MathCacheEntry::Pending => div()
            .py_2()
            .text_color(app.palette().muted)
            .child(app.tr(Msg::MathRendering)),
        MathCacheEntry::Error(error) => div()
            .py_2()
            .text_color(rgb(0xb91c1c))
            .child(app.math_error_message(&error)),
    };
    let payload_editor = div()
        .border_t_1()
        .border_color(rgb(0xe2e8f0))
        .bg(rgb(0xf8fafc))
        .p_2()
        .font(code_slot_font(&app.resolved_font_families.code))
        .text_size(px(typography.code_font_size))
        .line_height(px(typography.code_line_height))
        .child(visual_editor_field_element(
            app,
            block_index,
            payload,
            ElementId::from(("visual-math-payload", block.id.as_u64())),
            None,
            None,
            cx,
        ));
    div().child(visual_collapsible_source_block(
        app,
        block.id,
        payload,
        forced,
        true,
        presentation.into_any_element(),
        payload_editor.into_any_element(),
        cx,
    ))
}

fn visual_html_editor(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    html: &str,
    payload: &VisualEditorField,
    document_dir: Option<&Path>,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    let presentation = html_preview_block_view(app, html, block_index, document_dir, cx);
    let bordered = !html_preview_parts(html)
        .iter()
        .any(|part| matches!(part, HtmlPreviewPart::Table { .. }));
    let payload_editor = div()
        .border_t_1()
        .border_color(rgb(0xe2e8f0))
        .bg(rgb(0xf8fafc))
        .p_2()
        .font(code_slot_font(&app.resolved_font_families.code))
        .text_size(px(typography.source_island_font_size))
        .line_height(px(typography.source_island_line_height))
        .child(visual_editor_field_element(
            app,
            block_index,
            payload,
            ElementId::from(("visual-html-payload", block.id.as_u64())),
            None,
            None,
            cx,
        ));
    div().child(visual_collapsible_source_block(
        app,
        block.id,
        payload,
        false,
        bordered,
        presentation.into_any_element(),
        payload_editor.into_any_element(),
        cx,
    ))
}

/// Renders a diagram fence in Visual Edit by layering the rasterized diagram
/// on top of an on-demand source payload editor (collapsed by default; expand
/// via the hover `</>` control — same chrome as `visual_math_editor`).
///
/// The payload editor is the only editing path: it mutates the canonical
/// Markdown source via the normal visual-editor projection. The diagram image
/// is presentation-only; its completion or theme switch cannot rewrite the
/// fence (enforced by the diagram cache key scoping).
#[allow(clippy::too_many_arguments)]
fn visual_diagram_editor(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    language: Option<&str>,
    payload: &VisualEditorField,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    // Read the authored source through the same document slice the cache key
    // uses, so the visual-blocks render and the Split Preview render share one
    // cache entry for the same fence.
    let code = app.active_tab().document.text()[payload.source_range.clone()].to_string();
    let entry = app.diagram_entry(language, &code);
    let forced = !matches!(entry, Some(DiagramCacheEntry::Ready(_, _)));
    let presentation = match entry {
        Some(DiagramCacheEntry::Ready(image, size)) => {
            // Match Split Preview: pin intrinsic width so the supersampled
            // raster is presented at 1x, and let wide diagrams scroll
            // horizontally instead of being cropped or stretched. The scroll
            // wrapper needs an interactive id, matching the preview-math-scroll
            // pattern at `preview_block_view`.
            div().w_full().py_2().flex().justify_center().child(
                div()
                    .id(ElementId::from((
                        "visual-diagram-scroll",
                        block.id.as_u64(),
                    )))
                    .w_full()
                    .overflow_x_scroll()
                    .child(
                        div()
                            .min_w(size.width)
                            .flex()
                            .justify_center()
                            .child(img(ImageSource::Render(image)).w(size.width).max_w_full()),
                    ),
            )
        }
        Some(DiagramCacheEntry::Pending) => div()
            .w_full()
            .py_2()
            .text_color(app.palette().muted)
            .child(t(app.language, Msg::DiagramLoading)),
        Some(DiagramCacheEntry::Error(error)) => div()
            .w_full()
            .py_2()
            .text_color(rgb(0xb91c1c))
            .child(app.diagram_error_message(&error)),
        None => div()
            .w_full()
            .py_2()
            .text_color(app.palette().muted)
            .child(t(app.language, Msg::DiagramLoading)),
    };
    let payload_editor = div()
        .border_t_1()
        .border_color(rgb(0xe2e8f0))
        .bg(rgb(0xf8fafc))
        .p_2()
        .font(code_slot_font(&app.resolved_font_families.code))
        .text_size(px(typography.code_font_size))
        .line_height(px(typography.code_line_height))
        .child(code_block_header(
            app,
            language,
            code,
            code_palette(app.code_theme),
            cx,
        ))
        .child(visual_editor_field_element(
            app,
            block_index,
            payload,
            ElementId::from(("visual-diagram-payload", block.id.as_u64())),
            None,
            None,
            cx,
        ));
    div().child(visual_collapsible_source_block(
        app,
        block.id,
        payload,
        forced,
        true,
        presentation.into_any_element(),
        payload_editor.into_any_element(),
        cx,
    ))
}

/// Shared Obsidian-style chrome for block math and registered diagrams:
/// render-only by default, hover `</>` expands the payload editor, click
/// outside collapses (pending/error force the editor open).
fn visual_image_controls(
    offset: usize,
    presentation: ImagePresentation,
    language: Language,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let button = |id: &'static str, label: &'static str| {
        div()
            .id((id, offset))
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(0xcbd5e1))
            .bg(rgb(0xffffff))
            .text_size(px(11.))
            .cursor(CursorStyle::PointingHand)
            .child(label)
    };
    div()
        .mt_2()
        .flex()
        .flex_wrap()
        .gap_1()
        .child(
            button("image-width-25", "25%").on_click(cx.listener(move |app, _, _, cx| {
                app.set_image_presentation_at(
                    offset,
                    ImagePresentation {
                        width_percent: 25,
                        ..presentation
                    },
                    cx,
                )
            })),
        )
        .child(
            button("image-width-50", "50%").on_click(cx.listener(move |app, _, _, cx| {
                app.set_image_presentation_at(
                    offset,
                    ImagePresentation {
                        width_percent: 50,
                        ..presentation
                    },
                    cx,
                )
            })),
        )
        .child(
            button("image-width-75", "75%").on_click(cx.listener(move |app, _, _, cx| {
                app.set_image_presentation_at(
                    offset,
                    ImagePresentation {
                        width_percent: 75,
                        ..presentation
                    },
                    cx,
                )
            })),
        )
        .child(
            button("image-width-100", "100%").on_click(cx.listener(move |app, _, _, cx| {
                app.set_image_presentation_at(
                    offset,
                    ImagePresentation {
                        width_percent: 100,
                        ..presentation
                    },
                    cx,
                )
            })),
        )
        .child(
            button("image-align-left", p0_t(language, P0Msg::Left)).on_click(cx.listener(
                move |app, _, _, cx| {
                    app.set_image_presentation_at(
                        offset,
                        ImagePresentation {
                            alignment: ImageAlignment::Left,
                            ..presentation
                        },
                        cx,
                    )
                },
            )),
        )
        .child(
            button("image-align-center", p0_t(language, P0Msg::Center)).on_click(cx.listener(
                move |app, _, _, cx| {
                    app.set_image_presentation_at(
                        offset,
                        ImagePresentation {
                            alignment: ImageAlignment::Center,
                            ..presentation
                        },
                        cx,
                    )
                },
            )),
        )
        .child(
            button("image-align-right", p0_t(language, P0Msg::Right)).on_click(cx.listener(
                move |app, _, _, cx| {
                    app.set_image_presentation_at(
                        offset,
                        ImagePresentation {
                            alignment: ImageAlignment::Right,
                            ..presentation
                        },
                        cx,
                    )
                },
            )),
        )
        .child(
            button("image-replace", p0_t(language, P0Msg::Replace)).on_click(
                cx.listener(move |app, _, _, cx| app.replace_image_resource_at(offset, cx)),
            ),
        )
}

fn visual_collapsible_source_block(
    app: &MarkionApp,
    block_id: VisualBlockId,
    payload: &VisualEditorField,
    forced: bool,
    bordered: bool,
    presentation: gpui::AnyElement,
    payload_editor: gpui::AnyElement,
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    let tab = app.active_tab();
    let cursor = tab.cursor_offset();
    let caret_in_payload =
        payload.source_range.contains(&cursor) || cursor == payload.source_range.end;
    let user_expanded = tab.is_visual_source_expanded(block_id);
    let show_payload = forced || user_expanded || caret_in_payload;
    let show_toggle = forced
        || user_expanded
        || caret_in_payload
        || tab.hovered_visual_source_block == Some(block_id);

    div()
        .id(ElementId::from((
            "visual-collapsible-source",
            block_id.as_u64(),
        )))
        .relative()
        .mb_3()
        .when(bordered, |chrome| {
            chrome
                .border_1()
                .border_color(rgb(0xcbd5e1))
                .rounded_md()
                .overflow_hidden()
        })
        .on_hover(cx.listener(move |app, hovered: &bool, _, cx| {
            let tab = app.active_tab_mut();
            let next = hovered.then_some(block_id);
            let changed = if *hovered {
                tab.hovered_visual_source_block != Some(block_id)
            } else {
                tab.hovered_visual_source_block == Some(block_id)
            };
            if !changed {
                return;
            }
            tab.hovered_visual_source_block = next;
            cx.notify();
        }))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseDownEvent, _, _| {
                app.active_tab_mut().retain_visual_source_expand = Some(block_id);
            }),
        )
        .child(presentation)
        .when(show_payload, |chrome| chrome.child(payload_editor))
        .when(show_toggle, |chrome| {
            chrome.child(
                div()
                    .id(ElementId::from(("visual-source-toggle", block_id.as_u64())))
                    .absolute()
                    .top(px(4.))
                    .right(px(4.))
                    .px(px(6.))
                    .py(px(2.))
                    .rounded_sm()
                    .border_1()
                    .border_color(rgb(0xcbd5e1))
                    .bg(rgb(0xffffff))
                    .text_size(px(11.))
                    .line_height(px(14.))
                    .text_color(rgb(0x475569))
                    .cursor(CursorStyle::PointingHand)
                    .hover(|style| style.bg(rgb(0xf1f5f9)).text_color(rgb(0x0f172a)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |app, _: &MouseDownEvent, window, cx| {
                            cx.stop_propagation();
                            let focus_handle = app.focus_handle.clone();
                            window.focus(&focus_handle);
                            let tab = app.active_tab_mut();
                            tab.retain_visual_source_expand = Some(block_id);
                            if !forced {
                                tab.toggle_visual_source_expanded(block_id);
                                if tab.is_visual_source_expanded(block_id) {
                                    tab.expanded_visual_source_blocks
                                        .retain(|id| *id == block_id);
                                }
                            } else {
                                tab.set_visual_source_expanded(block_id, true);
                            }
                            cx.notify();
                        }),
                    )
                    .child("</>"),
            )
        })
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

pub(super) const VISUAL_TABLE_TOOLBAR_BUTTON_PADDING_X_PX: f32 = 6.;
pub(super) const VISUAL_TABLE_TOOLBAR_BUTTON_PADDING_Y_PX: f32 = 2.;
pub(super) const VISUAL_TABLE_TOOLBAR_BUTTON_FONT_SIZE_PX: f32 = 10.;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VisualTableToolbarTarget {
    pub(super) document_version: u64,
    pub(super) block_id: VisualBlockId,
    pub(super) row: usize,
    pub(super) column: usize,
    pub(super) source_offset: usize,
    pub(super) row_count: usize,
    pub(super) column_count: usize,
}

pub(super) fn visual_table_toolbar_target(
    document_version: u64,
    block: &VisualBlock,
    cursor_offset: usize,
) -> Option<VisualTableToolbarTarget> {
    let VisualBlockKind::Table { rows, .. } = &block.kind else {
        return None;
    };
    let Some(VisualBlockEditor::Table { cells }) = block.editor.as_ref() else {
        return None;
    };
    let row_count = rows.len();
    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    if row_count == 0 || column_count == 0 {
        return None;
    }

    let cell = cells.iter().find(|cell| {
        let range = &cell.field.source_range;
        cursor_offset >= range.start && cursor_offset <= range.end
    })?;
    if cell.row >= row_count || cell.column >= column_count {
        return None;
    }

    Some(VisualTableToolbarTarget {
        document_version,
        block_id: block.id,
        row: cell.row,
        column: cell.column,
        source_offset: cell.field.source_range.start,
        row_count,
        column_count,
    })
}

pub(super) fn table_toolbar_action_available(
    target: VisualTableToolbarTarget,
    edit: TableEdit,
) -> bool {
    match edit {
        TableEdit::Format | TableEdit::AddRow | TableEdit::AddColumn => true,
        TableEdit::DeleteRow => target.row > 0,
        TableEdit::MoveRowUp => target.row > 1,
        TableEdit::MoveRowDown => target.row > 0 && target.row + 1 < target.row_count,
        TableEdit::DeleteColumn => target.column_count > 1,
    }
}

pub(super) fn revalidate_visual_table_toolbar_target(
    target: VisualTableToolbarTarget,
    edit: TableEdit,
    document_version: u64,
    cursor_offset: usize,
    blocks: &[VisualBlock],
) -> Option<usize> {
    if target.document_version != document_version {
        return None;
    }
    let block = blocks.iter().find(|block| block.id == target.block_id)?;
    let current = visual_table_toolbar_target(document_version, block, cursor_offset)?;
    (current == target && table_toolbar_action_available(current, edit))
        .then_some(current.source_offset)
}

pub(super) fn visual_table_toolbar_is_visible(
    hovered: Option<VisualBlockId>,
    block_id: VisualBlockId,
    has_caret_target: bool,
) -> bool {
    hovered == Some(block_id) || has_caret_target
}

pub(super) fn visual_table_delete_available(
    blocks: &[VisualBlock],
    block_id: VisualBlockId,
) -> bool {
    blocks
        .iter()
        .position(|block| block.id == block_id)
        .is_some_and(|index| block_can_reorder_at(blocks, index))
}

pub(super) fn revalidate_visual_table_delete_target(
    target: BlockTarget,
    document_version: u64,
    blocks: &[VisualBlock],
) -> Option<BlockTarget> {
    let (index, _) = validate_block_target(document_version, blocks, &target).ok()?;
    block_can_reorder_at(blocks, index).then_some(target)
}

/// Flex recipe matching `.flex_1().min_w_0()` with a content-based grow weight.
/// GPUI's public `flex_grow()` helper only sets `1.0`, so the weight is applied
/// on the style refinement the same way other preview views poke `style.size`.
fn preview_table_cell_flex(weight: f32) -> Div {
    let mut cell = div().min_w_0();
    let style = cell.style();
    style.flex_grow = Some(weight.max(0.0));
    style.flex_shrink = Some(1.0);
    style.flex_basis = Some(gpui::relative(0.).into());
    cell
}

fn preview_table_column_weights(
    rows: &[Vec<RichText>],
    typography: &DocumentTypographyMetrics,
) -> Vec<f32> {
    table_column_flex_weights(rows, typography.table_font_size)
}

pub(super) fn visual_table_view(
    app: &MarkionApp,
    block: &VisualBlock,
    block_index: usize,
    rows: &[Vec<RichText>],
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    let typography = app.typography_metrics();
    let column_weights = preview_table_column_weights(rows, &typography);
    let table_offset = block.source_range.start;
    let block_id = block.id;
    let document_version = app.active_tab().document.version();
    let toolbar_target =
        visual_table_toolbar_target(document_version, block, app.active_tab().cursor_offset());
    let show_toolbar = visual_table_toolbar_is_visible(
        app.active_tab().hovered_visual_table_block,
        block_id,
        toolbar_target.is_some(),
    );
    let delete_target = BlockTarget::from_block(document_version, block);
    let delete_enabled =
        visual_table_delete_available(&app.active_tab().document.visual_blocks_shared(), block_id);
    let cells = match block.editor.as_ref() {
        Some(VisualBlockEditor::Table { cells }) => Some(cells),
        _ => None,
    };
    div()
        .id(ElementId::from(("visual-table", block_id.as_u64())))
        .debug_selector(move || format!("visual-table-chrome-{block_index}"))
        .mb_3()
        .border_1()
        .border_color(rgb(0xcbd5e1))
        .rounded_md()
        .overflow_hidden()
        .on_hover(cx.listener(move |app, hovered: &bool, _, cx| {
            let tab = app.active_tab_mut();
            let next = hovered.then_some(block_id);
            let changed = if *hovered {
                tab.hovered_visual_table_block != Some(block_id)
            } else {
                tab.hovered_visual_table_block == Some(block_id)
            };
            if !changed {
                return;
            }
            tab.hovered_visual_table_block = next;
            cx.notify();
        }))
        .when(show_toolbar, |table| {
            table.child(
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
                                let target = toolbar_target
                                    .filter(|target| table_toolbar_action_available(*target, edit));
                                preview_table_button(label, edit, status, target, cx)
                            }),
                    )
                    .child(preview_table_delete_button(
                        app.tr(Msg::VisualTableDeleteTable),
                        delete_enabled.then(|| delete_target.clone()),
                        cx,
                    )),
            )
        })
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
                    preview_table_cell_flex(column_weights.get(cell_index).copied().unwrap_or(1.0))
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
                        .child(if let Some(field) = field {
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
                                Some(cell),
                                cx,
                            )
                        } else {
                            rich_text_element(
                                app,
                                ElementId::from((
                                    "visual-table-cell-readonly",
                                    (block.id.as_u64() << 24)
                                        | ((row_index as u64) << 12)
                                        | cell_index as u64,
                                )),
                                cell,
                                block_index,
                                PreviewTextRunId::TableCell {
                                    row: row_index,
                                    col: cell_index,
                                },
                                cx,
                            )
                        })
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
                HtmlPreviewPart::Text {
                    text,
                    centered,
                    heading_level,
                    list_marker,
                    pre,
                    align,
                } => {
                    let font_size = heading_level
                        .map(|level| typography.heading_font_size(u32::from(level)))
                        .unwrap_or(if pre {
                            typography.code_font_size
                        } else {
                            typography.rendered_font_size
                        });
                    let line_height = if heading_level.is_some() {
                        font_size * 1.25
                    } else if pre {
                        typography.code_line_height
                    } else {
                        typography.paragraph_line_height
                    };
                    let marker_label = match list_marker {
                        Some(HtmlListMarker::Disc) => Some("•".to_string()),
                        Some(HtmlListMarker::Decimal(index)) => Some(format!("{index}.")),
                        None => None,
                    };
                    let has_marker = marker_label.is_some();
                    let body = rich_text_element(
                        app,
                        ElementId::from((
                            "preview-html-text",
                            ((block_index as u64) << 32) | part_index as u64,
                        )),
                        &text,
                        block_index,
                        PreviewTextRunId::HtmlText,
                        cx,
                    );
                    let aligned = match align {
                        HtmlAlign::Center if !has_marker => {
                            div().w_full().flex().justify_center().child(body)
                        }
                        HtmlAlign::End if !has_marker => {
                            div().w_full().flex().justify_end().child(body)
                        }
                        _ => div().child(body),
                    };
                    let content = if let Some(marker) = marker_label {
                        div()
                            .flex()
                            .gap_2()
                            .items_start()
                            .child(div().flex_none().child(marker))
                            .child(aligned)
                    } else {
                        aligned
                    };
                    div()
                        .mb_2()
                        .line_height(px(line_height))
                        .text_size(px(font_size))
                        .when(heading_level.is_some(), |style| {
                            style.font_weight(FontWeight::SEMIBOLD)
                        })
                        .when(pre, |style| {
                            style.font(code_slot_font(&app.resolved_font_families.code))
                        })
                        .when(centered && !has_marker, |style| style.text_center())
                        .child(content)
                }
                HtmlPreviewPart::Image {
                    url,
                    centered,
                    width,
                    height,
                    align,
                    ..
                } => div()
                    .mb_2()
                    .when(centered || align == HtmlAlign::Center, |style| {
                        style.flex().justify_center()
                    })
                    .when(align == HtmlAlign::End, |style| style.flex().justify_end())
                    .child(preview_image_view(app, &url, document_dir, width, height)),
                HtmlPreviewPart::Table { grid } => div().mb_2().child(html_table_grid_view(
                    app,
                    &grid,
                    block_index,
                    part_index,
                    document_dir,
                    &typography,
                    cx,
                )),
            }
        }))
}

/// Renders a resolved HTML table grid. Uses GPUI's CSS-grid layout so that
/// `colspan` / `rowspan` occupy exclusive start/end lines (`col_start`/`col_end`,
/// `row_start`/`row_end`). GPUI `col_span`/`row_span` must not be used here:
/// they wipe the start line and fall back to auto-placement. Column tracks
/// are weighted by cell content (browser auto-layout approximation) and
/// styling reuses the GFM pipe-table look (borders, header shading, table
/// font size) on cells that have visible content.
fn html_table_grid_view(
    app: &MarkionApp,
    grid: &HtmlTableGrid,
    block_index: usize,
    part_index: usize,
    document_dir: Option<&Path>,
    typography: &DocumentTypographyMetrics,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let columns = grid.columns.max(1).min(u16::MAX as usize) as u16;
    let border = rgb(0xe2e8f0);
    let outer_border = rgb(0xcbd5e1);
    let header_bg = rgb(0xf1f5f9);
    let body_bg = rgb(0xffffff);
    let font_size = typography.table_font_size;
    let padding = 8.0f32;

    // Walk the grid to assign explicit (column, row) coordinates to each
    // non-spacer cell. The grid model already advanced past rowspan-held
    // columns with spacer slots, so accumulating each row's col offsets while
    // skipping spacers yields the correct CSS-grid column line for every cell.
    let mut cell_views = Vec::new();
    let row_count = grid.rows.len();
    for (row_index, row) in grid.rows.iter().enumerate() {
        // Header emphasis is a per-row decision: an all-empty `<th>` frame
        // (Word-exported cover tables) renders as body cells, while a header
        // row with any visible content keeps every `<th>` emphasized,
        // including an empty matrix corner.
        let row_has_header = html_table_row_has_visible_header(row);
        let mut col: i16 = 1;
        for (cell_index, cell) in row.iter().enumerate() {
            if cell.is_spacer {
                col = col.saturating_add(cell.colspan.max(1) as i16);
                continue;
            }
            let colspan = cell.colspan.max(1).min(u16::MAX as usize) as u16;
            let rowspan = cell.rowspan.max(1).min(u16::MAX as usize) as u16;
            let col_start = col;
            let col_end = html_table_grid_line_end(col_start, colspan);
            let is_header = cell.is_header && row_has_header;
            let row_start = ((row_index + 1).min(i16::MAX as usize)) as i16;
            let row_end = html_table_grid_line_end(row_start, rowspan);
            let paint_empty = cell.image.is_none() && cell.content.text.trim().is_empty();
            // Internal grid lines: right border unless the cell touches the last
            // column, bottom border unless it touches the last row. The outer
            // container border draws the top/left/bottom/right edges.
            let touches_last_col = col_end > columns as i16;
            let touches_last_row = row_end as usize > row_count;
            let mut cell_view = div()
                .col_start(col_start)
                .col_end(col_end)
                .row_start(row_start)
                .row_end(row_end);
            if paint_empty {
                cell_views.push(cell_view);
            } else {
                cell_view = cell_view
                    .min_w_0()
                    .p(px(padding))
                    .text_size(px(font_size))
                    .when(is_header, |style| {
                        style.font_weight(FontWeight::SEMIBOLD).bg(header_bg)
                    })
                    .when(!is_header, |style| style.bg(body_bg))
                    .when(!touches_last_col, |style| {
                        style.border_r_1().border_color(border)
                    })
                    .when(!touches_last_row, |style| {
                        style.border_b_1().border_color(border)
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .min_w_0()
                            .children(cell.image.as_ref().map(|image| {
                                preview_image_view(
                                    app,
                                    &image.url,
                                    document_dir,
                                    image.width,
                                    image.height,
                                )
                            }))
                            .when(!cell.content.is_empty(), |style| {
                                style.child(rich_text_element(
                                    app,
                                    ElementId::from((
                                        "preview-html-table-cell",
                                        ((block_index as u64) << 40)
                                            | ((part_index as u64) << 28)
                                            | (((row_index as u64) & 0xff) << 14)
                                            | ((cell_index as u64) & 0x3fff),
                                    )),
                                    &cell.content,
                                    block_index,
                                    PreviewTextRunId::HtmlText,
                                    cx,
                                ))
                            }),
                    );
                cell_views.push(cell_view);
            }
            col = col.saturating_add(colspan as i16);
        }
    }
    div()
        .w_full()
        .grid()
        // Content-proportional column tracks approximate browser auto table
        // layout: empty spacer columns collapse to slivers instead of taking
        // an equal fraction of the table width.
        .grid_col_weights(html_table_column_weights(grid))
        .border_1()
        .border_color(outer_border)
        .rounded_md()
        .overflow_hidden()
        .children(cell_views)
}

/// Small "Copy" button rendered in the header of every code block. Clicking it
/// writes the block's raw text to the clipboard in one step, so users no longer
/// have to select the code manually. Works in preview, split, read, and Visual
/// Edit modes.
fn code_copy_button(
    app: &MarkionApp,
    code: String,
    palette: &CodePalette,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    div()
        .flex_none()
        .px_2()
        .py_1()
        .rounded_sm()
        .bg(palette.copy_bg)
        .text_color(palette.accent)
        .text_size(px(typography.small_font_size))
        .cursor_pointer()
        .child(t(app.language, Msg::ItemCopyCode))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(code.clone()));
                app.status = t(app.language, Msg::StatusCodeCopied).into();
                cx.notify();
            }),
        )
}

/// Header row for a code block: the optional language label on the left and the
/// one-click copy button on the right.
fn code_block_header(
    app: &MarkionApp,
    language: Option<&str>,
    code: String,
    palette: &CodePalette,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    div()
        .mb_2()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(typography.small_font_size))
                .text_color(palette.accent)
                .child(language.unwrap_or_default().to_string()),
        )
        .child(code_copy_button(app, code, palette, cx))
}

fn code_block_view(
    app: &MarkionApp,
    language: &Option<String>,
    code: &str,
    block_index: usize,
    show_code_line_numbers: bool,
    code_theme: CodeTheme,
    wrap_long_lines: bool,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    let palette = code_palette(code_theme);
    let highlighted = app.highlighted_code(language.as_deref(), code);
    let body = div()
        .mb_3()
        .p_3()
        .rounded_md()
        .bg(palette.bg)
        .text_color(palette.text)
        .font(code_slot_font(&app.resolved_font_families.code))
        .text_size(px(typography.code_font_size))
        .line_height(px(typography.code_line_height))
        .child(code_block_header(
            app,
            language.as_deref(),
            code.to_string(),
            palette,
            cx,
        ));
    if show_code_line_numbers {
        let rows = highlighted
            .iter()
            .enumerate()
            .map(|(line_index, line)| {
                let (styled, plain) = code_line_text(line, palette);
                // Wrapping keeps the line flexible so it soft-wraps; with
                // wrapping off the text keeps its intrinsic width so the
                // scroll container can host it without shrinking.
                let line_container = if wrap_long_lines {
                    div().flex_1().min_w_0()
                } else {
                    div().flex_none()
                };
                div()
                    .flex()
                    .items_start()
                    .min_w_full()
                    .child(
                        div()
                            .w(px(36.))
                            .flex_none()
                            .pr_2()
                            .text_color(palette.gutter)
                            .child(format!("{:>3}", line_index + 1)),
                    )
                    .child(line_container.child(selectable_plain_text(
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
            })
            .collect::<Vec<_>>();
        if wrap_long_lines {
            body.child(div().children(rows))
        } else {
            // A block child would cap at the container width, so the scroll
            // container is a flex row whose single `flex_none` child sizes to
            // its (nowrap) content — gpui derives the scroll extent from that
            // child's layout bounds.
            body.child(
                div()
                    .id(ElementId::from(("preview-code-scroll", block_index)))
                    .w_full()
                    .flex()
                    .overflow_x_scroll()
                    .child(
                        div()
                            .flex_none()
                            .min_w_full()
                            .whitespace_nowrap()
                            .flex()
                            .flex_col()
                            .children(rows),
                    ),
            )
        }
    } else {
        let (styled, plain) = code_block_text(&highlighted, palette);
        let code_text = div().min_w_full().child(selectable_plain_text(
            app,
            ElementId::from(("preview-code", block_index)),
            styled,
            plain,
            block_index,
            PreviewTextRunId::CodeBody,
            cx,
        ));
        if wrap_long_lines {
            body.child(code_text)
        } else {
            // Flex row + `flex_none` content, as in the gutter path above.
            body.child(
                div()
                    .id(ElementId::from(("preview-code-scroll", block_index)))
                    .w_full()
                    .flex()
                    .overflow_x_scroll()
                    .child(code_text.flex_none().whitespace_nowrap()),
            )
        }
    }
}

pub(super) fn preview_block_view(
    app: &MarkionApp,
    block: &PreviewBlock,
    block_index: usize,
    document_dir: Option<&Path>,
    show_code_line_numbers: bool,
    code_theme: CodeTheme,
    code_wrap: bool,
    display_scale: f32,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let typography = app.typography_metrics();
    match block {
        PreviewBlock::Heading { level, text, .. } => {
            let heading_size = typography.heading_font_size((*level).into());
            let size = px(heading_size);
            div()
                .mt_2()
                .mb_2()
                .text_size(size)
                .line_height(px(default_text_line_height(heading_size)))
                .font_weight(gpui::FontWeight::BOLD)
                .child(rich_text_with_math_element(
                    app,
                    "preview-heading",
                    text,
                    block_index,
                    PreviewTextRunId::Body,
                    display_scale,
                    heading_size,
                    default_text_line_height(heading_size),
                    document_dir,
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
                typography.rendered_font_size,
                typography.paragraph_line_height,
                document_dir,
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
                    typography.rendered_font_size,
                    typography.list_line_height,
                    document_dir,
                    cx,
                )))
        }
        PreviewBlock::BlockQuote { children, .. } => {
            let mut container = div()
                .mb_3()
                .pl_3()
                .border_l_1()
                .border_color(rgb(0x94a3b8))
                .text_color(rgb(0x475569))
                .text_size(px(typography.quote_font_size))
                .line_height(px(typography.quote_line_height));
            for (child_index, child) in children.iter().enumerate() {
                if let PreviewBlock::Paragraph { text, .. } = child {
                    if !text.is_empty() || text.spans.iter().any(|span| span.image.is_some()) {
                        container = container.child(rich_text_with_math_element(
                            app,
                            "preview-quote",
                            text,
                            block_index,
                            PreviewTextRunId::QuoteChild(child_index),
                            display_scale,
                            typography.quote_font_size,
                            typography.quote_line_height,
                            document_dir,
                            cx,
                        ));
                    }
                    continue;
                }
                if let PreviewBlock::Html { html, .. } = child {
                    container = container.child(html_preview_block_view(
                        app,
                        html,
                        block_index,
                        document_dir,
                        cx,
                    ));
                    continue;
                }
                let PreviewBlock::ListItem {
                    level,
                    ordered,
                    index,
                    checked,
                    text,
                    ..
                } = child
                else {
                    continue;
                };
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
                    _ => rgb(0x64748b),
                };
                container = container.child(
                    div()
                        .mt_1()
                        .ml(px((*level as f32 - 1.).max(0.) * 18.))
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
                            "preview-quote-list-item",
                            text,
                            block_index,
                            PreviewTextRunId::QuoteChild(child_index),
                            display_scale,
                            typography.quote_font_size,
                            typography.quote_line_height,
                            document_dir,
                            cx,
                        ))),
                );
            }
            container
        }
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
                        code_theme,
                        code_wrap,
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
                        code_theme,
                        code_wrap,
                        cx,
                    )),
                None => code_block_view(
                    app,
                    language,
                    code,
                    block_index,
                    show_code_line_numbers,
                    code_theme,
                    code_wrap,
                    cx,
                ),
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
                                    None,
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
        PreviewBlock::Image { url, .. } => {
            div()
                .mb_3()
                .child(preview_image_view(app, url, document_dir, None, None))
        }
        PreviewBlock::Rule { .. } => div().my_3().h(px(1.)).bg(rgb(0xcbd5e1)),
        PreviewBlock::FootnoteDefinition { label, text, .. } => div()
            .mb(px(typography.paragraph_spacing))
            .mt_2()
            .pt_2()
            .border_t_1()
            .border_color(rgb(0xe2e8f0))
            .flex()
            .items_start()
            .gap_2()
            .text_size(px(typography.rendered_font_size))
            .line_height(px(typography.paragraph_line_height))
            .child(
                div()
                    .flex_none()
                    .text_size(px(typography.rendered_font_size * 0.75))
                    .text_color(rgb(0x64748b))
                    .child(format!("[{label}]")),
            )
            .child(div().flex_1().min_w_0().child(rich_text_with_math_element(
                app,
                "preview-footnote",
                text,
                block_index,
                PreviewTextRunId::Body,
                display_scale,
                typography.rendered_font_size,
                typography.paragraph_line_height,
                document_dir,
                cx,
            ))),
        PreviewBlock::Table { rows, .. } => {
            // Split Preview and Read mode share this branch. Table mutation
            // belongs in Visual Edit or the source commands, so the preview
            // grid intentionally has no editing header or callbacks.
            let column_weights = preview_table_column_weights(rows, &typography);
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
                            preview_table_cell_flex(
                                column_weights.get(cell_index).copied().unwrap_or(1.0),
                            )
                            .p_2()
                            .when(!is_last_cell, |style| {
                                style.border_r_1().border_color(rgb(0xe2e8f0))
                            })
                            .text_size(px(typography.table_font_size))
                            .child(rich_text_element(
                                app,
                                ElementId::from((
                                    "preview-table-cell",
                                    ((block_index as u64) << 32)
                                        | (((row_index as u64) & 0xffff) << 16)
                                        | ((cell_index as u64) & 0xffff),
                                )),
                                cell,
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
    edit: TableEdit,
    status: Msg,
    target: Option<VisualTableToolbarTarget>,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let button = div()
        .flex_none()
        .px(px(VISUAL_TABLE_TOOLBAR_BUTTON_PADDING_X_PX))
        .py(px(VISUAL_TABLE_TOOLBAR_BUTTON_PADDING_Y_PX))
        .rounded_sm()
        .border_1()
        .text_size(px(VISUAL_TABLE_TOOLBAR_BUTTON_FONT_SIZE_PX))
        .child(label)
        .debug_selector(|| table_toolbar_action_debug_selector(edit, target.is_some()).to_string());

    let Some(target) = target else {
        return button
            .border_color(rgb(0xe2e8f0))
            .bg(rgb(0xf8fafc))
            .text_color(rgb(0x94a3b8));
    };

    button
        .border_color(rgb(0xcbd5e1))
        .bg(rgb(0xffffff))
        .text_color(rgb(0x334155))
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                window.focus(&app.focus_handle);
            }),
        )
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                cx.stop_propagation();
                let offset = {
                    let tab = app.active_tab();
                    revalidate_visual_table_toolbar_target(
                        target,
                        edit,
                        tab.document.version(),
                        tab.cursor_offset(),
                        &tab.visual_list_blocks,
                    )
                };
                if let Some(offset) = offset {
                    app.apply_table_edit_at(offset, edit, t(app.language, status).into(), cx);
                } else {
                    cx.notify();
                }
            }),
        )
}

pub(super) fn preview_table_delete_button(
    label: &'static str,
    target: Option<BlockTarget>,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let enabled = target.is_some();
    let button = div()
        .flex_none()
        .px(px(VISUAL_TABLE_TOOLBAR_BUTTON_PADDING_X_PX))
        .py(px(VISUAL_TABLE_TOOLBAR_BUTTON_PADDING_Y_PX))
        .rounded_sm()
        .border_1()
        .text_size(px(VISUAL_TABLE_TOOLBAR_BUTTON_FONT_SIZE_PX))
        .child(label)
        .debug_selector(move || {
            if enabled {
                "visual-table-delete-table".to_string()
            } else {
                "visual-table-delete-table-disabled".to_string()
            }
        });

    let Some(target) = target else {
        return button
            .border_color(rgb(0xe2e8f0))
            .bg(rgb(0xf8fafc))
            .text_color(rgb(0x94a3b8));
    };

    button
        .border_color(rgb(0xcbd5e1))
        .bg(rgb(0xffffff))
        .text_color(rgb(0x334155))
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                window.focus(&app.focus_handle);
            }),
        )
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                cx.stop_propagation();
                let confirmed = {
                    let tab = app.active_tab();
                    revalidate_visual_table_delete_target(
                        target.clone(),
                        tab.document.version(),
                        &tab.document.visual_blocks_shared(),
                    )
                };
                if let Some(target) = confirmed {
                    app.delete_visual_block(target, cx);
                } else {
                    cx.notify();
                }
            }),
        )
}

fn table_toolbar_action_debug_selector(edit: TableEdit, enabled: bool) -> &'static str {
    match (edit, enabled) {
        (TableEdit::Format, true) => "visual-table-format",
        (TableEdit::Format, false) => "visual-table-format-disabled",
        (TableEdit::AddRow, true) => "visual-table-add-row",
        (TableEdit::AddRow, false) => "visual-table-add-row-disabled",
        (TableEdit::DeleteRow, true) => "visual-table-delete-row",
        (TableEdit::DeleteRow, false) => "visual-table-delete-row-disabled",
        (TableEdit::MoveRowUp, true) => "visual-table-move-row-up",
        (TableEdit::MoveRowUp, false) => "visual-table-move-row-up-disabled",
        (TableEdit::MoveRowDown, true) => "visual-table-move-row-down",
        (TableEdit::MoveRowDown, false) => "visual-table-move-row-down-disabled",
        (TableEdit::AddColumn, true) => "visual-table-add-column",
        (TableEdit::AddColumn, false) => "visual-table-add-column-disabled",
        (TableEdit::DeleteColumn, true) => "visual-table-delete-column",
        (TableEdit::DeleteColumn, false) => "visual-table-delete-column-disabled",
    }
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

/// Full color set for rendered fenced code blocks: block chrome plus one
/// color per syntax token class. The Dark values are the historical look,
/// copied verbatim; the Light token colors mirror the PDF-export light
/// palette so a light code block matches its printed form.
pub(super) struct CodePalette {
    pub(super) bg: Rgba,
    pub(super) text: Rgba,
    pub(super) gutter: Rgba,
    /// Language label and copy-button accent.
    pub(super) accent: Rgba,
    pub(super) copy_bg: Rgba,
    plain: Rgba,
    keyword: Rgba,
    string: Rgba,
    number: Rgba,
    comment: Rgba,
    r#type: Rgba,
}

pub(super) const CODE_PALETTE_DARK: CodePalette = CodePalette {
    bg: Rgba {
        r: 0.058823529,
        g: 0.090196078,
        b: 0.16470588,
        a: 1.0,
    },
    text: Rgba {
        r: 0.88627451,
        g: 0.90980392,
        b: 0.94117647,
        a: 1.0,
    },
    gutter: Rgba {
        r: 0.39215686,
        g: 0.45490196,
        b: 0.54509804,
        a: 1.0,
    },
    accent: Rgba {
        r: 0.57647059,
        g: 0.77254902,
        b: 0.99215686,
        a: 1.0,
    },
    copy_bg: Rgba {
        r: 0.11764706,
        g: 0.16078431,
        b: 0.23137255,
        a: 1.0,
    },
    plain: Rgba {
        r: 0.88627451,
        g: 0.90980392,
        b: 0.94117647,
        a: 1.0,
    },
    keyword: Rgba {
        r: 0.75294118,
        g: 0.51764706,
        b: 0.98823529,
        a: 1.0,
    },
    string: Rgba {
        r: 0.5254902,
        g: 0.9372549,
        b: 0.6745098,
        a: 1.0,
    },
    number: Rgba {
        r: 0.98431373,
        g: 0.74901961,
        b: 0.14117647,
        a: 1.0,
    },
    comment: Rgba {
        r: 0.58039216,
        g: 0.63921569,
        b: 0.72156863,
        a: 1.0,
    },
    r#type: Rgba {
        r: 0.40392157,
        g: 0.90980392,
        b: 0.97647059,
        a: 1.0,
    },
};

pub(super) const CODE_PALETTE_LIGHT: CodePalette = CodePalette {
    bg: Rgba {
        r: 0.96470588,
        g: 0.97254902,
        b: 0.98039216,
        a: 1.0,
    },
    text: Rgba {
        r: 0.14117647,
        g: 0.16078431,
        b: 0.18431373,
        a: 1.0,
    },
    gutter: Rgba {
        r: 0.54901961,
        g: 0.58431373,
        b: 0.62352941,
        a: 1.0,
    },
    accent: Rgba {
        r: 0.035294118,
        g: 0.41176471,
        b: 0.85490196,
        a: 1.0,
    },
    copy_bg: Rgba {
        r: 0.91764706,
        g: 0.93333333,
        b: 0.94901961,
        a: 1.0,
    },
    plain: Rgba {
        r: 0.14117647,
        g: 0.16078431,
        b: 0.18431373,
        a: 1.0,
    },
    keyword: Rgba {
        r: 0.81176471,
        g: 0.13333333,
        b: 0.18039216,
        a: 1.0,
    },
    string: Rgba {
        r: 0.058823529,
        g: 0.4627451,
        b: 0.16862745,
        a: 1.0,
    },
    number: Rgba {
        r: 0.035294118,
        g: 0.33333333,
        b: 0.64705882,
        a: 1.0,
    },
    comment: Rgba {
        r: 0.43137255,
        g: 0.46666667,
        b: 0.50196078,
        a: 1.0,
    },
    r#type: Rgba {
        r: 0.0,
        g: 0.43921569,
        b: 0.56470588,
        a: 1.0,
    },
};

pub(super) fn code_palette(theme: CodeTheme) -> &'static CodePalette {
    match theme {
        CodeTheme::Dark => &CODE_PALETTE_DARK,
        CodeTheme::Light => &CODE_PALETTE_LIGHT,
    }
}

impl CodePalette {
    pub(super) fn token_color(&self, kind: HighlightKind) -> Rgba {
        match kind {
            HighlightKind::Plain => self.plain,
            HighlightKind::Keyword => self.keyword,
            HighlightKind::String => self.string,
            HighlightKind::Number => self.number,
            HighlightKind::Comment => self.comment,
            HighlightKind::Type => self.r#type,
        }
    }
}

pub(super) fn utf16_offset_to_byte_offset(text: &str, offset: usize) -> Option<usize> {
    let mut utf16_count = 0;

    for (byte_offset, ch) in text.char_indices() {
        if utf16_count == offset {
            return Some(byte_offset);
        }
        utf16_count += ch.len_utf16();
        if utf16_count > offset {
            return None;
        }
    }

    (utf16_count == offset).then_some(text.len())
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
