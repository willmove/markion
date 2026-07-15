use super::*;

const BOUNDARY_SCAN_WINDOW: usize = 1024;

/// Where a grapheme scan for the cluster around `offset` may safely start:
/// the current line start (segmentation restarts after every hard break), or
/// the nearest char boundary [`BOUNDARY_SCAN_WINDOW`] bytes back when the
/// line itself is longer than that.
pub(super) fn boundary_scan_start(text: &str, offset: usize) -> usize {
    let mut window_start = offset.saturating_sub(BOUNDARY_SCAN_WINDOW);
    while !text.is_char_boundary(window_start) {
        window_start += 1;
    }
    text[window_start..offset]
        .rfind('\n')
        .map_or(window_start, |idx| window_start + idx + 1)
}

/// Cache key for the editor's measured wrapped height (see
/// `EditorTab::measured_height_cache`): the height only changes when one of
/// these inputs does.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct MeasuredHeightKey {
    pub(super) version: u64,
    pub(super) wrap_width: Pixels,
    pub(super) font_size: Pixels,
    pub(super) line_height: Pixels,
}

#[derive(Clone)]
pub(super) struct EditorSnapshot {
    pub(super) document: MarkdownDocument,
    pub(super) selected_range: Range<usize>,
    pub(super) selection_reversed: bool,
}

/// One entry in the undo/redo history.
///
/// Edit sites push `Full` pre-edit snapshots exactly as before, but
/// [`push_history_entry`] compacts the previously-newest full entry into a
/// `Diff` at push time — the only moment both texts are in hand. Each stack
/// therefore retains at most one whole-document copy (its newest entry) no
/// matter how long the history grows; previously a 1 MB document accumulated
/// up to `MAX_HISTORY_LEN` full clones (~200 MB) while typing.
#[allow(clippy::large_enum_variant)]
pub(super) enum UndoEntry {
    Full(EditorSnapshot),
    Diff(UndoDiff),
}

/// Compact history record. LIFO order guarantees that when this entry is
/// popped the document text is exactly the state the diff was computed
/// against, so applying it means: replace `range` of the current text with
/// `insert`, then restore the recorded selection.
pub(super) struct UndoDiff {
    pub(super) range: Range<usize>,
    pub(super) insert: String,
    pub(super) selected_range: Range<usize>,
    pub(super) selection_reversed: bool,
}

/// Push a history entry onto `stack`, compacting the previous top into a
/// [`UndoDiff`] when both it and the new entry are `Full` (a `Diff` on top is
/// already compact, and its presence means the buried full entry pops against
/// a text we cannot know yet). Caps the stack at [`MAX_HISTORY_LEN`].
pub(super) fn push_history_entry(stack: &mut Vec<UndoEntry>, entry: UndoEntry) {
    if let (UndoEntry::Full(new), Some(top)) = (&entry, stack.last_mut())
        && let UndoEntry::Full(old) = top
    {
        *top = UndoEntry::Diff(compact_history_entry(old, new.document.text()));
    }
    stack.push(entry);
    if stack.len() > MAX_HISTORY_LEN {
        stack.remove(0);
    }
}

/// Compact `older` (a full snapshot) into a diff against `newer_text`, the
/// state that will be current when the entry is popped: replacing the changed
/// byte range of `newer_text` with the bytes it replaced in `older` restores
/// the older text exactly.
pub(super) fn compact_history_entry(older: &EditorSnapshot, newer_text: &str) -> UndoDiff {
    let old_text = older.document.text();
    let old_bytes = old_text.as_bytes();
    let new_bytes = newer_text.as_bytes();
    let max_prefix = old_bytes.len().min(new_bytes.len());
    let mut prefix = 0;
    while prefix < max_prefix && old_bytes[prefix] == new_bytes[prefix] {
        prefix += 1;
    }
    let max_suffix = max_prefix - prefix;
    let mut suffix = 0;
    while suffix < max_suffix
        && old_bytes[old_bytes.len() - 1 - suffix] == new_bytes[new_bytes.len() - 1 - suffix]
    {
        suffix += 1;
    }
    // The byte-level bounds may fall inside a UTF-8 sequence when the old and
    // new text share leading/trailing bytes of different chars (e.g. 中 vs 串
    // share two of three bytes); widen to char boundaries in both strings so
    // the stored slices stay valid UTF-8.
    while prefix > 0 && (!old_text.is_char_boundary(prefix) || !newer_text.is_char_boundary(prefix))
    {
        prefix -= 1;
    }
    while suffix > 0
        && (!old_text.is_char_boundary(old_text.len() - suffix)
            || !newer_text.is_char_boundary(newer_text.len() - suffix))
    {
        suffix -= 1;
    }
    UndoDiff {
        range: prefix..newer_text.len() - suffix,
        insert: old_text[prefix..old_text.len() - suffix].to_string(),
        selected_range: older.selected_range.clone(),
        selection_reversed: older.selection_reversed,
    }
}

/// Per-document editor state. One open document per tab; `MarkionApp` holds a
/// `Vec<EditorTab>` + an `active_tab` index. All cursor/scroll/undo/selection
/// state lives here so it is isolated per document. Per-window state (menus,
/// themes, sidebar, search panel) stays on `MarkionApp`.
pub(super) struct EditorTab {
    pub(super) document: MarkdownDocument,
    pub(super) undo_stack: Vec<UndoEntry>,
    pub(super) redo_stack: Vec<UndoEntry>,
    pub(super) editor_scroll: ScrollHandle,
    /// Virtualized preview: GPUI's `list` renders only the blocks intersecting
    /// the viewport (+overdraw), so preview cost is O(visible blocks) instead of
    /// O(document). The state is intrusive and must persist across frames.
    pub(super) preview_list: ListState,
    pub(super) visual_list: ListState,
    pub(super) visual_list_blocks: std::sync::Arc<Vec<VisualBlock>>,
    /// Snapshot of the block slice `preview_list` currently reflects. Each frame
    /// we diff the freshly-parsed blocks against this and `splice` only the
    /// changed range into `preview_list`, which preserves scroll position (a
    /// full `reset` would jump to the top on every keystroke).
    pub(super) preview_list_blocks: std::sync::Arc<Vec<PreviewBlock>>,
    /// Debounced preview parsing (Split/Read): the latest document version a
    /// render has observed, and the version the preview blocks actually
    /// reflect. When they differ the preview is stale and a parse is due once
    /// the debounce window elapses (or `PREVIEW_MAX_STALE` forces one).
    pub(super) preview_seen_version: u64,
    pub(super) preview_reflects_version: Option<u64>,
    /// When the document last changed / was last parsed for the preview, used
    /// to decide "typing has settled" and "too stale, parse anyway".
    pub(super) preview_changed_at: Option<Instant>,
    pub(super) preview_reflects_at: Option<Instant>,
    /// Generation token incremented whenever a new debounce timer is armed (or
    /// the pending one must be cancelled); a firing timer compares its captured
    /// generation against this and does nothing if it lost the race.
    pub(super) preview_debounce_generation: u64,
    /// Id of the background preview parse currently in flight for this tab
    /// (`next_preview_parse_id`), or `None`. At most one parse runs per tab;
    /// ids are globally unique so a landing result can find its owning tab by
    /// id (tab indices shift when other tabs close) and a result whose tab was
    /// replaced meanwhile (`reset_preview_list` clears the marker) is dropped.
    pub(super) preview_parse_inflight: Option<u64>,
    pub(super) selected_range: Range<usize>,
    pub(super) selection_reversed: bool,
    pub(super) marked_range: Option<Range<usize>>,
    pub(super) last_lines: Vec<WrappedLine>,
    pub(super) line_offsets: Vec<usize>,
    pub(super) line_heights: Vec<Pixels>,
    pub(super) last_bounds: Option<Bounds<Pixels>>,
    /// Actual line height from the last layout pass, reused by hit-testing so
    /// mouse positions line up with the painted text.
    pub(super) line_height: Pixels,
    pub(super) is_selecting: bool,
    /// The document text as a `SharedString`, cached per document version so
    /// the editor element does not copy the whole document on every frame.
    pub(super) display_text_cache: RefCell<Option<(u64, SharedString)>>,
    /// Total wrapped height from the last layout measure. The measure closure
    /// runs on every layout pass and a full-document `shape_text` — even one
    /// that hits GPUI's per-line layout cache — still walks and hashes every
    /// line; this memo makes repeat measures O(1). (`text_version` values are
    /// globally unique, so a replaced document can never alias a stale entry.)
    pub(super) measured_height_cache: RefCell<Option<(MeasuredHeightKey, Pixels)>>,
    /// Byte offset of each logical line start, cached per document version:
    /// prepaint needs the table every frame and rebuilding it is an
    /// O(document) `match_indices` scan.
    pub(super) line_offsets_cache: RefCell<Option<(u64, Rc<Vec<usize>>)>>,
    pub(super) last_recovery_file: Option<PathBuf>,
    /// Generation token incremented on every autosave schedule; a pending timer
    /// compares its captured generation against this to decide whether to fire.
    pub(super) autosave_generation: u64,
    /// Last scroll fraction applied to the editor during sync reconciliation
    /// (None until the first reconciled Split frame). Used to detect which pane
    /// drove the latest scroll change so only the *other* pane is written.
    pub(super) sync_scroll_editor_fraction: Option<f32>,
    /// Last scroll fraction applied to the preview during sync reconciliation.
    pub(super) sync_scroll_preview_fraction: Option<f32>,
    /// Active drag/copy selection in the rendered preview for this tab.
    /// Independent of the source editor selection; never mutates the document.
    pub(super) preview_selection: Option<PreviewSelection>,
    /// True while the user is dragging a preview text selection.
    pub(super) preview_is_selecting: bool,
}

impl EditorTab {
    pub(super) fn new(document: MarkdownDocument) -> Self {
        let version = document.version();
        Self {
            document,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            editor_scroll: ScrollHandle::new(),
            preview_list: ListState::new(0, ListAlignment::Top, px(PREVIEW_LIST_OVERDRAW)),
            visual_list: ListState::new(0, ListAlignment::Top, px(PREVIEW_LIST_OVERDRAW)),
            visual_list_blocks: std::sync::Arc::new(Vec::new()),
            preview_list_blocks: std::sync::Arc::new(Vec::new()),
            // Seen = current version so the first render is not mistaken for an
            // edit; reflects = None so that same render parses immediately.
            preview_seen_version: version,
            preview_reflects_version: None,
            preview_changed_at: None,
            preview_reflects_at: None,
            preview_debounce_generation: 0,
            preview_parse_inflight: None,
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_lines: Vec::new(),
            line_offsets: Vec::new(),
            line_heights: Vec::new(),
            last_bounds: None,
            line_height: px(EDITOR_LINE_HEIGHT),
            is_selecting: false,
            display_text_cache: RefCell::new(None),
            measured_height_cache: RefCell::new(None),
            line_offsets_cache: RefCell::new(None),
            last_recovery_file: None,
            autosave_generation: 0,
            sync_scroll_editor_fraction: None,
            sync_scroll_preview_fraction: None,
            preview_selection: None,
            preview_is_selecting: false,
        }
    }

    /// Bring `preview_list` in line with a freshly-computed block slice.
    ///
    /// The heavy `preview_blocks_shared()` cache returns the *same* `Arc` when
    /// the document has not changed, so the pointer-equality fast path makes an
    /// unchanged frame free. When the content differs we compute the minimal
    /// changed block range (common prefix/suffix) and `splice` only that range,
    /// which keeps the list's scroll position anchored instead of snapping to
    /// the top the way `reset` would.
    pub(super) fn sync_preview_list(&mut self, blocks: &std::sync::Arc<Vec<PreviewBlock>>) {
        if std::sync::Arc::ptr_eq(&self.preview_list_blocks, blocks) {
            return;
        }
        let (range, count) = preview_block_splice(&self.preview_list_blocks, blocks);
        if !range.is_empty() || count != 0 {
            self.preview_list.splice(range, count);
        }
        self.preview_list_blocks = blocks.clone();
        self.preview_selection =
            invalidate_preview_selection_if_stale(self.preview_selection.take(), blocks.len());
        if self.preview_selection.is_none() {
            self.preview_is_selecting = false;
        }
    }

    pub(super) fn sync_visual_list(&mut self, blocks: &std::sync::Arc<Vec<VisualBlock>>) {
        if std::sync::Arc::ptr_eq(&self.visual_list_blocks, blocks) {
            return;
        }
        let (range, count) = block_splice(&self.visual_list_blocks, blocks);
        if !range.is_empty() || count != 0 {
            self.visual_list.splice(range, count);
        }
        self.visual_list_blocks = blocks.clone();
    }

    /// Drop the preview list back to an empty, top-scrolled state. Used when the
    /// document is wholesale replaced (open/new/reload) so the next render
    /// rebuilds the list from scratch and starts at the top rather than
    /// inheriting the previous document's scroll offset.
    pub(super) fn reset_preview_list(&mut self) {
        self.preview_list.reset(0);
        self.preview_list_blocks = std::sync::Arc::new(Vec::new());
        // Reset the debounce so the replacement document parses on its next
        // render rather than waiting out a debounce window, and invalidate any
        // pending timer armed for the old document.
        self.preview_seen_version = self.document.version();
        self.preview_reflects_version = None;
        self.preview_changed_at = None;
        self.preview_reflects_at = None;
        self.preview_debounce_generation = self.preview_debounce_generation.wrapping_add(1);
        // Orphan any in-flight background parse: its result belongs to the
        // replaced document and must not be applied to this one.
        self.preview_parse_inflight = None;
        self.visual_list.reset(0);
        self.visual_list_blocks = std::sync::Arc::new(Vec::new());
        // The replacement document's scroll ranges differ, so the cached sync
        // fractions are stale; let the next Split frame re-derive them.
        self.sync_scroll_editor_fraction = None;
        self.sync_scroll_preview_fraction = None;
        self.clear_preview_selection();
    }

    pub(super) fn clear_preview_selection(&mut self) {
        self.preview_selection = None;
        self.preview_is_selecting = false;
    }

    /// Cached `SharedString` copy of the document text for the current
    /// version. Cloning the returned value is an `Arc` bump, not a text copy.
    pub(super) fn shared_document_text(&self) -> SharedString {
        let version = self.document.version();
        if let Some((cached_version, text)) = self.display_text_cache.borrow().as_ref()
            && *cached_version == version
        {
            return text.clone();
        }
        let text: SharedString = self.document.text().to_string().into();
        *self.display_text_cache.borrow_mut() = Some((version, text.clone()));
        text
    }

    /// Byte offset at the start of each logical line, cached per document
    /// version. Cloning the returned value is an `Rc` bump.
    pub(super) fn shared_line_offsets(&self) -> Rc<Vec<usize>> {
        let version = self.document.version();
        if let Some((cached_version, offsets)) = self.line_offsets_cache.borrow().as_ref()
            && *cached_version == version
        {
            return offsets.clone();
        }
        let text = self.document.text();
        let offsets = Rc::new(
            std::iter::once(0)
                .chain(text.match_indices('\n').map(|(i, _)| i + 1))
                .collect::<Vec<usize>>(),
        );
        *self.line_offsets_cache.borrow_mut() = Some((version, offsets.clone()));
        offsets
    }

    pub(super) fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub(super) fn scroll_editor_to_offset(&self, offset: usize) {
        let offset = clamp_to_text_boundary(self.document.text(), offset);
        let line = self.document.text()[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        self.editor_scroll
            .set_offset(point(px(0.), -px(line as f32 * EDITOR_LINE_HEIGHT)));
    }

    pub(super) fn scroll_editor_typewriter_to_offset(&self, offset: usize) {
        let offset = clamp_to_text_boundary(self.document.text(), offset);
        let line = self.document.text()[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        // Keep the caret ~10 lines below the viewport top ("typewriter" band).
        let y = (line as f32 * EDITOR_LINE_HEIGHT - 10. * EDITOR_LINE_HEIGHT).max(0.);
        self.editor_scroll.set_offset(point(px(0.), -px(y)));
    }

    pub(super) fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            document: self.document.clone(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
        }
    }

    pub(super) fn push_undo_snapshot(&mut self) {
        self.commit_undo_snapshot(self.snapshot());
    }

    pub(super) fn commit_undo_snapshot(&mut self, snapshot: EditorSnapshot) {
        push_history_entry(&mut self.undo_stack, UndoEntry::Full(snapshot));
        self.redo_stack.clear();
    }

    /// Restore a full snapshot's document and selection.
    pub(super) fn restore_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.document = snapshot.document;
        self.document.refresh_dirty_from_disk();
        self.selected_range = snapshot.selected_range;
        self.selection_reversed = snapshot.selection_reversed;
        self.marked_range = None;
    }

    /// Apply a compact history record and return its inverse — the record
    /// that, pushed onto the opposite stack, re-creates the state being left.
    pub(super) fn apply_history_diff(&mut self, diff: UndoDiff) -> UndoDiff {
        let inverse = UndoDiff {
            range: diff.range.start..diff.range.start + diff.insert.len(),
            insert: self.document.text()[diff.range.clone()].to_string(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
        };
        self.document.replace_range(diff.range, &diff.insert);
        self.document.refresh_dirty_from_disk();
        self.selected_range = diff.selected_range;
        self.selection_reversed = diff.selection_reversed;
        self.marked_range = None;
        inverse
    }

    /// Pop and apply the newest undo entry, pushing its inverse onto the redo
    /// stack. Returns false when there is nothing to undo.
    pub(super) fn apply_undo(&mut self) -> bool {
        let Some(entry) = self.undo_stack.pop() else {
            return false;
        };
        match entry {
            UndoEntry::Full(snapshot) => {
                let current = self.snapshot();
                push_history_entry(&mut self.redo_stack, UndoEntry::Full(current));
                self.restore_snapshot(snapshot);
            }
            UndoEntry::Diff(diff) => {
                let inverse = self.apply_history_diff(diff);
                push_history_entry(&mut self.redo_stack, UndoEntry::Diff(inverse));
            }
        }
        true
    }

    /// Pop and apply the newest redo entry, pushing its inverse onto the undo
    /// stack (without clearing redo). Returns false when there is nothing to
    /// redo.
    pub(super) fn apply_redo(&mut self) -> bool {
        let Some(entry) = self.redo_stack.pop() else {
            return false;
        };
        match entry {
            UndoEntry::Full(snapshot) => {
                let current = self.snapshot();
                push_history_entry(&mut self.undo_stack, UndoEntry::Full(current));
                self.restore_snapshot(snapshot);
            }
            UndoEntry::Diff(diff) => {
                let inverse = self.apply_history_diff(diff);
                push_history_entry(&mut self.undo_stack, UndoEntry::Diff(inverse));
            }
        }
        true
    }

    pub(super) fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.document.text().is_empty() {
            return 0;
        }

        let (Some(bounds), true) = (self.last_bounds.as_ref(), !self.last_lines.is_empty()) else {
            return self.document.text().len();
        };

        let local_y = position.y - bounds.top();
        if local_y < px(0.) {
            return 0;
        }

        // Find the WrappedLine containing this y position, accounting for wrap.
        let mut line_index = 0;
        let mut cumulative_y = px(0.);
        for (i, &height) in self.line_heights.iter().enumerate() {
            let next_y = cumulative_y + height;
            if local_y >= cumulative_y && local_y < next_y {
                line_index = i;
                break;
            }
            cumulative_y = next_y;
            line_index = i;
        }

        let line = &self.last_lines[line_index];
        let local_y_in_line = local_y - cumulative_y;
        let local_point = point(position.x - bounds.left(), local_y_in_line);
        let line_byte_offset = match line.closest_index_for_position(local_point, self.line_height)
        {
            Ok(idx) | Err(idx) => idx,
        };

        let line_start = *self
            .line_offsets
            .get(line_index)
            .unwrap_or(&self.document.text().len());
        (line_start + line_byte_offset).min(self.document.text().len())
    }

    /// Translate a document byte offset to a screen-space point within `bounds`,
    /// resolving which logical line it belongs to and asking that line's layout
    /// for the wrapped position.
    pub(super) fn layout_point_for_offset(
        &self,
        offset: usize,
        bounds: Bounds<Pixels>,
        line_height: Pixels,
    ) -> Option<Point<Pixels>> {
        if self.last_lines.is_empty() || self.line_offsets.is_empty() {
            return Some(point(bounds.left(), bounds.top()));
        }
        let text_len = self.document.text().len();
        let clamped = offset.min(text_len);
        // Find the logical line containing this offset.
        let mut line_index = self.line_offsets.len() - 1;
        for (i, &start) in self.line_offsets.iter().enumerate() {
            if clamped >= start {
                line_index = i;
            } else {
                break;
            }
        }
        let line_start = self.line_offsets[line_index];
        let local_offset = clamped - line_start;
        let line = self.last_lines.get(line_index)?;
        let local = line.position_for_index(local_offset, line_height)?;
        let mut cumulative_y = px(0.);
        for i in 0..line_index {
            cumulative_y += self.line_heights.get(i).copied().unwrap_or(line_height);
        }
        Some(point(
            bounds.left() + local.x,
            bounds.top() + cumulative_y + local.y,
        ))
    }

    /// Start of the grapheme cluster preceding `offset`.
    ///
    /// Grapheme segmentation restarts at every hard line break (the only
    /// cluster containing one is "\r\n", handled explicitly), so scanning from
    /// the current line start gives the same boundary as segmenting the whole
    /// document — the previous implementation did exactly that and cost an
    /// O(document) walk per Backspace / arrow key (~1ms on a 1 MB document).
    pub(super) fn previous_boundary(&self, offset: usize) -> usize {
        let text = self.document.text();
        let offset = offset.min(text.len());
        if offset == 0 {
            return 0;
        }
        let scan_start = boundary_scan_start(text, offset);
        if scan_start == offset {
            // The cursor sits right after a line break: the preceding cluster
            // is the break itself, and "\r\n" is a single two-byte cluster.
            return if offset >= 2 && text.as_bytes()[offset - 2] == b'\r' {
                offset - 2
            } else {
                offset - 1
            };
        }
        text[scan_start..offset]
            .grapheme_indices(true)
            .next_back()
            .map(|(idx, _)| scan_start + idx)
            .unwrap_or(scan_start)
    }

    /// Start of the grapheme cluster following `offset` (the first boundary
    /// strictly greater than it). Scans from the current line start; see
    /// [`Self::previous_boundary`].
    pub(super) fn next_boundary(&self, offset: usize) -> usize {
        let text = self.document.text();
        if offset >= text.len() {
            return text.len();
        }
        let scan_start = boundary_scan_start(text, offset);
        text[scan_start..]
            .grapheme_indices(true)
            .map(|(idx, _)| scan_start + idx)
            .find(|&idx| idx > offset)
            .unwrap_or(text.len())
    }

    pub(super) fn offset_from_utf16(&self, offset: usize) -> usize {
        utf16_offset_to_byte_offset(self.document.text(), offset)
    }

    pub(super) fn offset_to_utf16(&self, offset: usize) -> usize {
        byte_offset_to_utf16_offset(self.document.text(), offset)
    }

    pub(super) fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub(super) fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    pub(super) fn relative_range_from_utf16(
        text: &str,
        range_utf16: &Range<usize>,
    ) -> Range<usize> {
        utf16_offset_to_byte_offset(text, range_utf16.start)
            ..utf16_offset_to_byte_offset(text, range_utf16.end)
    }
}

pub(super) fn comparable_document_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub(super) fn path_is_within_workspace(root: &Path, path: &Path) -> bool {
    comparable_document_path(path).starts_with(comparable_document_path(root))
}

pub(super) fn workspace_root_for_document(
    current_root: Option<&Path>,
    document_path: &Path,
) -> Option<PathBuf> {
    if let Some(root) = current_root.filter(|root| path_is_within_workspace(root, document_path)) {
        return Some(comparable_document_path(root));
    }

    document_path.parent().map(comparable_document_path)
}

pub(super) fn scan_result_matches_workspace(requested_root: &Path, current_root: &Path) -> bool {
    comparable_document_path(requested_root) == comparable_document_path(current_root)
}

pub(super) fn workspace_root_needs_reset(
    current_root: &Path,
    has_file_tree: bool,
    next_root: &Path,
) -> bool {
    !has_file_tree || !scan_result_matches_workspace(current_root, next_root)
}

pub(super) fn update_file_tree_collapse_state_from_scan(
    scanned: &io::Result<FileTree>,
    collapsed_paths: &mut HashSet<PathBuf>,
    needs_initial_collapse: &mut bool,
) {
    let Ok(tree) = scanned else {
        return;
    };

    if *needs_initial_collapse {
        *collapsed_paths = tree
            .entries
            .iter()
            .filter(|entry| entry.depth == 0 && entry.kind == FileTreeEntryKind::Directory)
            .map(|entry| entry.path.clone())
            .collect();
        *needs_initial_collapse = false;
    } else {
        collapsed_paths.retain(|path| path.exists());
    }
}

pub(super) fn open_folder_prompt_options(language: Language) -> PathPromptOptions {
    PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some(t(language, Msg::PromptOpenFolder).into()),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum StartupOpenIntent {
    None,
    File(PathBuf),
    Folder(PathBuf),
    Invalid {
        path: PathBuf,
        reason: StartupOpenInvalidReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StartupOpenInvalidReason {
    Missing,
    UnsupportedFile,
    UnsupportedPath,
}

impl StartupOpenInvalidReason {
    fn label(self) -> &'static str {
        match self {
            Self::Missing => "path does not exist",
            Self::UnsupportedFile => "unsupported file type",
            Self::UnsupportedPath => "path is not a file or folder",
        }
    }
}

impl StartupOpenIntent {
    pub(super) fn from_env_args() -> Self {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::from_args(env::args_os().skip(1), &cwd)
    }

    pub(super) fn from_args<I>(args: I, cwd: &Path) -> Self
    where
        I: IntoIterator<Item = OsString>,
    {
        let Some(path) = args.into_iter().next().map(PathBuf::from) else {
            return Self::None;
        };
        Self::from_path(resolve_startup_path(path, cwd))
    }

    pub(super) fn from_path(path: PathBuf) -> Self {
        if path.is_file() {
            if is_markdown_path(&path) {
                Self::File(path)
            } else {
                Self::Invalid {
                    path,
                    reason: StartupOpenInvalidReason::UnsupportedFile,
                }
            }
        } else if path.is_dir() {
            Self::Folder(path)
        } else if !path.exists() {
            Self::Invalid {
                path,
                reason: StartupOpenInvalidReason::Missing,
            }
        } else {
            Self::Invalid {
                path,
                reason: StartupOpenInvalidReason::UnsupportedPath,
            }
        }
    }
}

pub(super) fn resolve_startup_path(path: PathBuf, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

pub(super) fn startup_open_failure_detail(path: &Path, reason: StartupOpenInvalidReason) -> String {
    format!("{} ({})", path.display(), reason.label())
}

pub(super) fn find_tab_with_document_path(tabs: &[EditorTab], path: &Path) -> Option<usize> {
    let target = comparable_document_path(path);
    tabs.iter().position(|tab| {
        tab.document
            .path()
            .is_some_and(|open_path| comparable_document_path(open_path) == target)
    })
}
