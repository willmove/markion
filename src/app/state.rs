use super::*;

const BOUNDARY_SCAN_WINDOW: usize = 1024;
pub(super) const SEMANTIC_UNDO_TIMEOUT: Duration = Duration::from_millis(900);

/// Stable, session-local identity for one outline heading. Source offsets are
/// deliberately excluded: inserting body text before a heading must not unfold
/// it. The full ancestor path and duplicate ordinal distinguish equal titles.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct OutlineNodeKey(Vec<OutlineNodeSegment>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct OutlineNodeSegment {
    level: u8,
    title: String,
    same_named_sibling_ordinal: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OutlineProjectedRow {
    pub(super) outline_index: usize,
    pub(super) key: OutlineNodeKey,
    pub(super) has_children: bool,
    pub(super) collapsed: bool,
    pub(super) active: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct OutlineProjection {
    pub(super) rows: Vec<OutlineProjectedRow>,
}

struct OutlineKeyFrame {
    level: u8,
    key: OutlineNodeKey,
    child_counts: HashMap<(u8, String), usize>,
}

/// Derive semantic node keys from the cached flat outline in one forward scan.
/// A heading's parent is the nearest preceding heading at a shallower level;
/// skipped levels therefore need no synthetic nodes.
pub(super) fn outline_node_keys(headings: &[markion::Heading]) -> Vec<OutlineNodeKey> {
    let mut keys = Vec::with_capacity(headings.len());
    let mut stack: Vec<OutlineKeyFrame> = Vec::new();
    let mut root_counts: HashMap<(u8, String), usize> = HashMap::new();

    for heading in headings {
        while stack
            .last()
            .is_some_and(|ancestor| ancestor.level >= heading.level)
        {
            stack.pop();
        }

        let count_key = (heading.level, heading.title.clone());
        let ordinal = if let Some(parent) = stack.last_mut() {
            let next = parent.child_counts.entry(count_key).or_default();
            let ordinal = *next;
            *next += 1;
            ordinal
        } else {
            let next = root_counts.entry(count_key).or_default();
            let ordinal = *next;
            *next += 1;
            ordinal
        };

        let mut path = stack
            .last()
            .map(|parent| parent.key.0.clone())
            .unwrap_or_default();
        path.push(OutlineNodeSegment {
            level: heading.level,
            title: heading.title.clone(),
            same_named_sibling_ordinal: ordinal,
        });
        let key = OutlineNodeKey(path);
        keys.push(key.clone());
        stack.push(OutlineKeyFrame {
            level: heading.level,
            key,
            child_counts: HashMap::new(),
        });
    }

    keys
}

struct OutlineVisibilityFrame {
    level: u8,
    /// First collapsed ancestor. Because every later ancestor is hidden under
    /// it, this is also the nearest collapsed ancestor that is actually visible.
    hidden_by: Option<usize>,
}

fn project_outline_rows_with_keys(
    headings: &[markion::Heading],
    keys: &[OutlineNodeKey],
    collapsed: &HashSet<OutlineNodeKey>,
    current: Option<usize>,
) -> OutlineProjection {
    debug_assert_eq!(headings.len(), keys.len());
    let mut rows = Vec::with_capacity(headings.len());
    let mut ancestors: Vec<OutlineVisibilityFrame> = Vec::new();
    let mut active_representative = None;

    for (index, (heading, key)) in headings.iter().zip(keys).enumerate() {
        while ancestors
            .last()
            .is_some_and(|ancestor| ancestor.level >= heading.level)
        {
            ancestors.pop();
        }

        let hidden_by = ancestors.last().and_then(|ancestor| ancestor.hidden_by);
        let has_children = headings
            .get(index + 1)
            .is_some_and(|next| next.level > heading.level);
        let is_collapsed = has_children && collapsed.contains(key);

        if current == Some(index) {
            active_representative = Some(hidden_by.unwrap_or(index));
        }
        if hidden_by.is_none() {
            rows.push(OutlineProjectedRow {
                outline_index: index,
                key: key.clone(),
                has_children,
                collapsed: is_collapsed,
                active: false,
            });
        }

        ancestors.push(OutlineVisibilityFrame {
            level: heading.level,
            hidden_by: hidden_by.or_else(|| is_collapsed.then_some(index)),
        });
    }

    if let Some(active_index) = active_representative {
        if let Some(row) = rows
            .iter_mut()
            .find(|row| row.outline_index == active_index)
        {
            row.active = true;
        }
    }

    OutlineProjection { rows }
}

#[cfg(test)]
pub(super) fn project_outline_rows(
    headings: &[markion::Heading],
    collapsed: &HashSet<OutlineNodeKey>,
    current: Option<usize>,
) -> OutlineProjection {
    let keys = outline_node_keys(headings);
    project_outline_rows_with_keys(headings, &keys, collapsed, current)
}

fn outline_key_is_unambiguous(key: &OutlineNodeKey, keys: &[OutlineNodeKey]) -> bool {
    key.0.iter().enumerate().all(|(depth, segment)| {
        keys.iter()
            .filter(|candidate| {
                candidate.0.len() > depth
                    && candidate.0[..depth] == key.0[..depth]
                    && candidate.0[depth].level == segment.level
                    && candidate.0[depth].title == segment.title
            })
            .count()
            == 1
    })
}

/// Retain folds across body-only edits and unambiguous hierarchy changes.
/// Ambiguous duplicate groups are kept when the structural key sequence is
/// unchanged, but are conservatively unfolded when an edit changes that
/// sequence so an inserted equal-title sibling cannot inherit another row's
/// fold by ordinal.
pub(super) fn reconcile_outline_collapsed_keys(
    collapsed: &mut HashSet<OutlineNodeKey>,
    previous_keys: &[OutlineNodeKey],
    current_keys: &[OutlineNodeKey],
) {
    if previous_keys == current_keys {
        return;
    }

    let live_keys: HashSet<&OutlineNodeKey> = current_keys.iter().collect();
    collapsed.retain(|key| {
        live_keys.contains(key)
            && outline_key_is_unambiguous(key, previous_keys)
            && outline_key_is_unambiguous(key, current_keys)
    });
}

#[derive(Debug, Default)]
pub(super) struct OutlineFoldingState {
    observed_document_version: Option<u64>,
    known_keys: Vec<OutlineNodeKey>,
    collapsed: HashSet<OutlineNodeKey>,
}

impl OutlineFoldingState {
    fn projection(
        &mut self,
        document_version: u64,
        headings: &[markion::Heading],
        current: Option<usize>,
    ) -> OutlineProjection {
        let keys = outline_node_keys(headings);
        if self.observed_document_version != Some(document_version) {
            reconcile_outline_collapsed_keys(&mut self.collapsed, &self.known_keys, &keys);
            self.known_keys = keys.clone();
            self.observed_document_version = Some(document_version);
        }
        project_outline_rows_with_keys(headings, &keys, &self.collapsed, current)
    }

    fn toggle(&mut self, key: OutlineNodeKey) -> Option<bool> {
        if !self.known_keys.contains(&key) {
            return None;
        }
        if self.collapsed.remove(&key) {
            Some(false)
        } else {
            self.collapsed.insert(key);
            Some(true)
        }
    }

    #[cfg(test)]
    pub(super) fn collapsed_keys(&self) -> &HashSet<OutlineNodeKey> {
        &self.collapsed
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VisualNavigationCaret {
    pub(super) source_offset: usize,
    pub(super) x: Pixels,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct VisualNavigationLine {
    pub(super) y: Pixels,
    pub(super) carets: Vec<VisualNavigationCaret>,
}

impl VisualNavigationLine {
    pub(super) fn closest_source(&self, preferred_x: Pixels) -> Option<usize> {
        self.carets
            .iter()
            .min_by(|left, right| {
                (left.x - preferred_x)
                    .abs()
                    .to_f64()
                    .total_cmp(&(right.x - preferred_x).abs().to_f64())
            })
            .map(|caret| caret.source_offset)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct VisualNavigationSnapshot {
    pub(super) document_version: u64,
    pub(super) block_index: usize,
    pub(super) source_selection: Range<usize>,
    pub(super) marked_range: Option<Range<usize>>,
    pub(super) source_island: bool,
    pub(super) lines: Vec<VisualNavigationLine>,
}

impl VisualNavigationSnapshot {
    pub(super) fn line_index_for_source(&self, source: usize) -> Option<usize> {
        self.lines
            .iter()
            .position(|line| {
                line.carets
                    .iter()
                    .any(|caret| caret.source_offset == source)
            })
            .or_else(|| {
                self.lines
                    .iter()
                    .enumerate()
                    .filter_map(|(index, line)| {
                        let distance = line
                            .carets
                            .iter()
                            .map(|caret| caret.source_offset.abs_diff(source))
                            .min()?;
                        Some((index, distance))
                    })
                    .min_by_key(|(_, distance)| *distance)
                    .map(|(index, _)| index)
            })
    }

    pub(super) fn caret_x_for_source(&self, source: usize) -> Option<Pixels> {
        let line = self.lines.get(self.line_index_for_source(source)?)?;
        line.carets
            .iter()
            .min_by_key(|caret| caret.source_offset.abs_diff(source))
            .map(|caret| caret.x)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VisualNavigationDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PendingVisualNavigation {
    pub(super) document_version: u64,
    pub(super) target_block: usize,
    pub(super) direction: VisualNavigationDirection,
    pub(super) extend_selection: bool,
    pub(super) preferred_x: Pixels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VisualNavigationPosition {
    pub(super) document_version: u64,
    pub(super) block_index: usize,
    pub(super) line_index: usize,
    pub(super) source_offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UndoCaptureKind {
    Insert,
    Delete,
    Ime,
    Atomic,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct UndoCapture {
    pub(super) kind: UndoCaptureKind,
    pub(super) last_edit_at: Instant,
    pub(super) next_cursor: usize,
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

/// One content tab in the workspace. Documents retain the existing editor
/// state wholesale, while image tabs carry only their read-only presentation
/// state and can therefore never acquire document dirty/undo/recovery state.
pub(super) enum WorkspaceTab {
    Document(DocumentTabState),
    Image(ImageTabState),
}

/// Presentation-only identity for the source editor geometry used by
/// source-mapped Split Preview scrolling. The shaped lines themselves remain
/// in `DocumentTabState`; this key prevents geometry from an older document or
/// wrap width from being used after an edit or reflow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SourceLayoutKey {
    pub(super) version: u64,
    pub(super) wrap_width: Pixels,
    pub(super) line_height: Pixels,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct SyncPreviewPosition {
    pub(super) item_ix: usize,
    pub(super) offset_in_item: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ExpectedSyncFollower {
    Editor(f32),
    Preview(SyncPreviewPosition),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PendingPreviewRefinement {
    pub(super) version: u64,
    pub(super) item_ix: usize,
    pub(super) progress: f32,
}

/// Per-tab observation and intent state for source-mapped scroll coupling.
/// Scroll handles/list state remain independent; this only distinguishes user
/// movement from the follower writes produced by reconciliation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct SyncScrollState {
    pub(super) last_editor_offset: Option<f32>,
    pub(super) last_preview_position: Option<SyncPreviewPosition>,
    pub(super) driver_hint: Option<PaneScrollTarget>,
    pub(super) deferred_driver: Option<PaneScrollTarget>,
    pub(super) expected_follower: Option<ExpectedSyncFollower>,
    pub(super) expected_follower_retried: bool,
    pub(super) pending_preview_refinement: Option<PendingPreviewRefinement>,
}

impl SyncScrollState {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn mark_driver(&mut self, driver: PaneScrollTarget) {
        self.driver_hint = Some(driver);
        self.deferred_driver = None;
        self.expected_follower = None;
        self.expected_follower_retried = false;
        self.pending_preview_refinement = None;
    }

    pub(super) fn invalidate_geometry(&mut self) {
        let deferred = self.driver_hint.or(self.deferred_driver);
        self.last_editor_offset = None;
        self.last_preview_position = None;
        self.driver_hint = None;
        self.deferred_driver = deferred;
        self.expected_follower = None;
        self.expected_follower_retried = false;
        self.pending_preview_refinement = None;
    }
}

/// Compatibility name used throughout the existing document-oriented code.
/// The actual tab vector stores the heterogeneous [`WorkspaceTab`] sum type.
pub(super) type EditorTab = WorkspaceTab;

pub(super) struct ImageTabState {
    pub(super) path: PathBuf,
    pub(super) key: PreviewImageKey,
    pub(super) scroll: ScrollHandle,
    pub(super) claimed: bool,
}

impl ImageTabState {
    pub(super) fn presentation_memory_bytes(&self) -> usize {
        self.path.as_os_str().len()
            + std::mem::size_of::<PreviewImageKey>()
            + std::mem::size_of::<ScrollHandle>()
            + std::mem::size_of::<bool>()
    }
}

/// Per-document editor state. All cursor/scroll/undo/selection and derived
/// Markdown cache state remains isolated here and is absent from image tabs.
pub(super) struct DocumentTabState {
    pub(super) document: MarkdownDocument,
    /// Destination divergence detected after open/save. This state never
    /// rewrites the canonical in-memory Markdown by itself.
    pub(super) external_conflict: Option<DiskState>,
    pub(super) recovery_id: u64,
    pub(super) undo_stack: Vec<UndoEntry>,
    pub(super) redo_stack: Vec<UndoEntry>,
    pub(super) undo_capture: Option<UndoCapture>,
    pub(super) pending_text_edit_intent: Option<UndoCaptureKind>,
    pub(super) editor_scroll: ScrollHandle,
    /// Session-only outline tree presentation. Interior mutability lets the
    /// render path reconcile semantic keys without touching document state.
    pub(super) outline_folding: RefCell<OutlineFoldingState>,
    /// Virtualized preview: GPUI's `list` renders only the blocks intersecting
    /// the viewport (+overdraw), so preview cost is O(visible blocks) instead of
    /// O(document). The state is intrusive and must persist across frames.
    pub(super) preview_list: ListState,
    pub(super) visual_list: ListState,
    pub(super) visual_list_blocks: std::sync::Arc<Vec<VisualBlock>>,
    /// Preview-image keys currently claimed by this tab in `PreviewImageCache`.
    /// Refreshed when preview/visual lists sync; released on tab close/replace.
    pub(super) claimed_preview_images: HashSet<PreviewImageKey>,
    /// Presentation-only: block-math / diagram fences whose LaTeX/source pane
    /// the user expanded via the hover `</>` control. Not persisted; pruned when
    /// stable block ids disappear after a visual-list sync.
    pub(super) expanded_visual_source_blocks: HashSet<VisualBlockId>,
    /// Block currently under the pointer for showing the source-toggle control.
    pub(super) hovered_visual_source_block: Option<VisualBlockId>,
    /// Set by a collapsible block's mouse-down so the Visual Edit surface can
    /// collapse other expanded blocks without collapsing the clicked one.
    pub(super) retain_visual_source_expand: Option<VisualBlockId>,
    /// One-shot request consumed by the next Visual Edit render. Keeping this
    /// separate from list state avoids snapping manual scroll back to the caret
    /// on every unrelated frame.
    pub(super) visual_cursor_reveal_pending: bool,
    /// Ephemeral screen-space geometry produced by the focused visual row.
    pub(super) visual_caret_bounds: Option<Bounds<Pixels>>,
    pub(super) visual_marked_range_bounds: Option<(Range<usize>, Bounds<Pixels>)>,
    /// Disambiguates a caret at a display boundary shared by the two source
    /// sides of hidden Markdown syntax. This is interaction state only.
    pub(super) visual_caret_affinity: Option<VisualCaretAffinity>,
    pub(super) visual_caret_affinity_version: Option<u64>,
    pub(super) visual_navigation_snapshots: HashMap<usize, VisualNavigationSnapshot>,
    pub(super) visual_navigation_snapshot_ids: HashMap<usize, VisualBlockId>,
    pub(super) visual_preferred_x: Option<Pixels>,
    pub(super) visual_navigation_position: Option<VisualNavigationPosition>,
    pub(super) pending_visual_navigation: Option<PendingVisualNavigation>,
    /// Bounds of the Visual Edit input bridge, used as an IME fallback before
    /// the focused virtual row has painted.
    pub(super) visual_input_bounds: Option<Bounds<Pixels>>,
    #[cfg(test)]
    pub(super) visual_last_projection: Option<(String, Vec<Range<usize>>)>,
    #[cfg(test)]
    pub(super) visual_last_projection_styles: Option<Vec<InlineStyle>>,
    #[cfg(test)]
    pub(super) visual_projection_paint_count: usize,
    #[cfg(test)]
    pub(super) visual_caret_paint_count: usize,
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
    /// Prefix Y positions for `last_lines`, including the trailing total.
    /// Together with the shaped lines and offsets this is the versioned source
    /// layout snapshot used by semantic scroll mapping.
    pub(super) line_tops: Vec<Pixels>,
    pub(super) source_layout_key: Option<SourceLayoutKey>,
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
    pub(super) sync_scroll_state: SyncScrollState,
    /// Active drag/copy selection in the rendered preview for this tab.
    /// Independent of the source editor selection; never mutates the document.
    pub(super) preview_selection: Option<PreviewSelection>,
    /// True while the user is dragging a preview text selection.
    pub(super) preview_is_selecting: bool,
}

impl WorkspaceTab {
    pub(super) fn new(document: MarkdownDocument) -> Self {
        Self::Document(DocumentTabState::new(document))
    }

    pub(super) fn new_image(path: PathBuf, key: PreviewImageKey) -> Self {
        Self::Image(ImageTabState {
            path,
            key,
            scroll: ScrollHandle::new(),
            claimed: false,
        })
    }

    pub(super) fn path(&self) -> Option<&Path> {
        match self {
            Self::Document(tab) => tab.document.path(),
            Self::Image(image) => Some(&image.path),
        }
    }

    pub(super) fn title(&self) -> String {
        title_from_path(self.path()).to_string()
    }

    pub(super) fn focus_identity(&self) -> Option<PathBuf> {
        self.path().map(comparable_document_path)
    }

    pub(super) fn is_image(&self) -> bool {
        matches!(self, Self::Image(_))
    }

    pub(super) fn is_document(&self) -> bool {
        matches!(self, Self::Document(_))
    }

    pub(super) fn is_dirty(&self) -> bool {
        self.document_tab()
            .is_some_and(|tab| tab.document.is_dirty())
    }

    pub(super) fn requires_discard_confirmation(&self) -> bool {
        self.is_dirty()
    }

    pub(super) fn document_tab(&self) -> Option<&DocumentTabState> {
        match self {
            Self::Document(tab) => Some(tab),
            Self::Image(_) => None,
        }
    }

    pub(super) fn document_tab_mut(&mut self) -> Option<&mut DocumentTabState> {
        match self {
            Self::Document(tab) => Some(tab),
            Self::Image(_) => None,
        }
    }

    pub(super) fn image(&self) -> Option<&ImageTabState> {
        match self {
            Self::Image(image) => Some(image),
            Self::Document(_) => None,
        }
    }

    pub(super) fn image_mut(&mut self) -> Option<&mut ImageTabState> {
        match self {
            Self::Image(image) => Some(image),
            Self::Document(_) => None,
        }
    }

    pub(super) fn enter_dormant(&mut self) -> HashSet<PreviewImageKey> {
        match self {
            Self::Document(tab) => tab.enter_dormant(),
            Self::Image(image) => {
                if std::mem::take(&mut image.claimed) {
                    HashSet::from([image.key.clone()])
                } else {
                    HashSet::new()
                }
            }
        }
    }

    /// Release only decoded-image cache claims, preserving every document
    /// cache and presentation state. Used when a document is covered by an
    /// image tab so switching back is state-neutral.
    pub(super) fn take_image_claims(&mut self) -> HashSet<PreviewImageKey> {
        match self {
            Self::Document(tab) => std::mem::take(&mut tab.claimed_preview_images),
            Self::Image(image) => {
                if std::mem::take(&mut image.claimed) {
                    HashSet::from([image.key.clone()])
                } else {
                    HashSet::new()
                }
            }
        }
    }

    pub(super) fn relative_range_from_utf16(
        text: &str,
        range_utf16: &Range<usize>,
    ) -> Option<Range<usize>> {
        DocumentTabState::relative_range_from_utf16(text, range_utf16)
    }
}

impl std::ops::Deref for WorkspaceTab {
    type Target = DocumentTabState;

    fn deref(&self) -> &Self::Target {
        self.document_tab()
            .expect("document-only state accessed while an image tab is active")
    }
}

impl std::ops::DerefMut for WorkspaceTab {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.document_tab_mut()
            .expect("document-only state accessed while an image tab is active")
    }
}

impl DocumentTabState {
    fn new(document: MarkdownDocument) -> Self {
        let version = document.version();
        Self {
            document,
            external_conflict: None,
            recovery_id: next_recovery_id(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_capture: None,
            pending_text_edit_intent: None,
            editor_scroll: ScrollHandle::new(),
            outline_folding: RefCell::new(OutlineFoldingState::default()),
            preview_list: ListState::new(0, ListAlignment::Top, px(PREVIEW_LIST_OVERDRAW)),
            visual_list: ListState::new(0, ListAlignment::Top, px(PREVIEW_LIST_OVERDRAW)),
            visual_list_blocks: std::sync::Arc::new(Vec::new()),
            claimed_preview_images: HashSet::new(),
            expanded_visual_source_blocks: HashSet::new(),
            hovered_visual_source_block: None,
            retain_visual_source_expand: None,
            visual_cursor_reveal_pending: false,
            visual_caret_bounds: None,
            visual_marked_range_bounds: None,
            visual_caret_affinity: None,
            visual_caret_affinity_version: None,
            visual_navigation_snapshots: HashMap::new(),
            visual_navigation_snapshot_ids: HashMap::new(),
            visual_preferred_x: None,
            visual_navigation_position: None,
            pending_visual_navigation: None,
            visual_input_bounds: None,
            #[cfg(test)]
            visual_last_projection: None,
            #[cfg(test)]
            visual_last_projection_styles: None,
            #[cfg(test)]
            visual_projection_paint_count: 0,
            #[cfg(test)]
            visual_caret_paint_count: 0,
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
            line_tops: Vec::new(),
            source_layout_key: None,
            last_bounds: None,
            line_height: px(EDITOR_LINE_HEIGHT),
            is_selecting: false,
            display_text_cache: RefCell::new(None),
            measured_height_cache: RefCell::new(None),
            line_offsets_cache: RefCell::new(None),
            last_recovery_file: None,
            autosave_generation: 0,
            sync_scroll_state: SyncScrollState::default(),
            preview_selection: None,
            preview_is_selecting: false,
        }
    }

    pub(super) fn outline_projection(
        &self,
        headings: &[markion::Heading],
        current: Option<usize>,
    ) -> OutlineProjection {
        self.outline_folding
            .borrow_mut()
            .projection(self.document.version(), headings, current)
    }

    pub(super) fn toggle_outline_node(&self, key: OutlineNodeKey) -> Option<bool> {
        self.outline_folding.borrow_mut().toggle(key)
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
        self.sync_scroll_state.invalidate_geometry();
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
        let (range, count) = visual_block_splice(&self.visual_list_blocks, blocks);
        if !range.is_empty() || count != 0 {
            self.visual_list.splice(range, count);
        }
        self.visual_list_blocks = blocks.clone();
        let live_ids: HashSet<VisualBlockId> = blocks.iter().map(|block| block.id).collect();
        self.expanded_visual_source_blocks
            .retain(|id| live_ids.contains(id));
        if self
            .hovered_visual_source_block
            .is_some_and(|id| !live_ids.contains(&id))
        {
            self.hovered_visual_source_block = None;
        }
        if self
            .retain_visual_source_expand
            .is_some_and(|id| !live_ids.contains(&id))
        {
            self.retain_visual_source_expand = None;
        }
        self.visual_navigation_snapshots.retain(|index, snapshot| {
            snapshot.document_version == self.document.version()
                && self.visual_navigation_snapshot_ids.get(index)
                    == blocks.get(*index).map(|block| &block.id)
        });
        self.visual_navigation_snapshot_ids
            .retain(|index, id| blocks.get(*index).is_some_and(|block| block.id == *id));
    }

    pub(super) fn take_visual_cursor_reveal_index(
        &mut self,
        blocks: &[VisualBlock],
    ) -> Option<usize> {
        if !std::mem::take(&mut self.visual_cursor_reveal_pending) {
            return None;
        }
        visual_block_index_for_offset(blocks, self.cursor_offset(), self.document.text().len())
    }

    pub(super) fn is_visual_source_expanded(&self, block_id: VisualBlockId) -> bool {
        self.expanded_visual_source_blocks.contains(&block_id)
    }

    pub(super) fn toggle_visual_source_expanded(&mut self, block_id: VisualBlockId) {
        if !self.expanded_visual_source_blocks.remove(&block_id) {
            self.expanded_visual_source_blocks.insert(block_id);
        }
    }

    pub(super) fn set_visual_source_expanded(&mut self, block_id: VisualBlockId, expanded: bool) {
        if expanded {
            self.expanded_visual_source_blocks.insert(block_id);
        } else {
            self.expanded_visual_source_blocks.remove(&block_id);
        }
    }

    /// After a Visual Edit pointer down, keep `retain` expanded (if any) and
    /// collapse every other manually expanded source pane.
    pub(super) fn apply_visual_source_outside_click(&mut self) {
        let retain = self.retain_visual_source_expand.take();
        self.expanded_visual_source_blocks
            .retain(|id| retain == Some(*id));
    }

    /// Enter inactive-tab dormancy: drop expensive derived/layout caches while
    /// retaining text, selection, undo/redo, and scroll handles.
    ///
    /// Returns preview-image keys that were claimed by this tab so the app can
    /// release them from `PreviewImageCache` (same path as tab close).
    pub(super) fn enter_dormant(&mut self) -> HashSet<PreviewImageKey> {
        self.document.evict_derived_caches();
        self.invalidate_source_layout();
        *self.display_text_cache.borrow_mut() = None;
        *self.measured_height_cache.borrow_mut() = None;
        *self.line_offsets_cache.borrow_mut() = None;

        self.preview_list.reset(0);
        self.preview_list_blocks = std::sync::Arc::new(Vec::new());
        self.preview_reflects_version = None;
        self.preview_changed_at = None;
        self.preview_reflects_at = None;
        self.preview_debounce_generation = self.preview_debounce_generation.wrapping_add(1);
        self.preview_parse_inflight = None;
        self.preview_seen_version = self.document.version();
        self.clear_preview_selection();

        self.visual_list.reset(0);
        self.visual_list_blocks = std::sync::Arc::new(Vec::new());
        self.expanded_visual_source_blocks.clear();
        self.hovered_visual_source_block = None;
        self.retain_visual_source_expand = None;
        self.visual_caret_bounds = None;
        self.visual_marked_range_bounds = None;
        self.clear_visual_caret_affinity();
        self.clear_visual_navigation_intent();
        self.visual_navigation_snapshots.clear();
        self.visual_navigation_snapshot_ids.clear();
        self.visual_input_bounds = None;
        #[cfg(test)]
        {
            self.visual_last_projection = None;
            self.visual_last_projection_styles = None;
            self.visual_projection_paint_count = 0;
            self.visual_caret_paint_count = 0;
        }

        // Scroll handles are retained for reactivation; observations are
        // reseeded after the source/preview geometry is rebuilt.
        std::mem::take(&mut self.claimed_preview_images)
    }

    /// Drop the preview list back to an empty, top-scrolled state. Used when the
    /// document is wholesale replaced (open/new/reload) so the next render
    /// rebuilds the list from scratch and starts at the top rather than
    /// inheriting the previous document's scroll offset.
    pub(super) fn reset_preview_list(&mut self) {
        self.preview_list.reset(0);
        self.preview_list_blocks = std::sync::Arc::new(Vec::new());
        *self.outline_folding.borrow_mut() = OutlineFoldingState::default();
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
        // Claims are released by MarkionApp before reset; clear the local set
        // so a replaced tab never thinks it still owns those keys.
        self.claimed_preview_images.clear();
        self.expanded_visual_source_blocks.clear();
        self.hovered_visual_source_block = None;
        self.retain_visual_source_expand = None;
        self.visual_cursor_reveal_pending = true;
        self.visual_caret_bounds = None;
        self.visual_marked_range_bounds = None;
        self.clear_visual_caret_affinity();
        self.clear_visual_navigation_intent();
        self.visual_navigation_snapshots.clear();
        self.visual_navigation_snapshot_ids.clear();
        self.visual_input_bounds = None;
        #[cfg(test)]
        {
            self.visual_last_projection = None;
            self.visual_last_projection_styles = None;
            self.visual_projection_paint_count = 0;
            self.visual_caret_paint_count = 0;
        }
        self.sync_scroll_state.reset();
        self.clear_preview_selection();
    }

    pub(super) fn invalidate_source_layout(&mut self) {
        self.last_lines.clear();
        self.line_offsets.clear();
        self.line_heights.clear();
        self.line_tops.clear();
        self.source_layout_key = None;
        self.last_bounds = None;
        self.sync_scroll_state.reset();
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

    pub(super) fn clear_visual_caret_affinity(&mut self) {
        self.visual_caret_affinity = None;
        self.visual_caret_affinity_version = None;
    }

    pub(super) fn set_visual_caret_affinity(&mut self, affinity: Option<VisualCaretAffinity>) {
        self.visual_caret_affinity = affinity;
        self.visual_caret_affinity_version = affinity.map(|_| self.document.version());
    }

    pub(super) fn current_visual_caret_affinity(&self) -> Option<VisualCaretAffinity> {
        (self.visual_caret_affinity_version == Some(self.document.version()))
            .then_some(self.visual_caret_affinity)
            .flatten()
    }

    pub(super) fn clear_visual_navigation_intent(&mut self) {
        self.visual_preferred_x = None;
        self.visual_navigation_position = None;
        self.pending_visual_navigation = None;
    }

    pub(super) fn register_visual_navigation_snapshot(
        &mut self,
        mut snapshot: VisualNavigationSnapshot,
    ) {
        let Some(block_id) = self
            .visual_list_blocks
            .get(snapshot.block_index)
            .map(|block| block.id)
        else {
            return;
        };
        self.visual_navigation_snapshots
            .retain(|_, existing| existing.document_version == snapshot.document_version);
        self.visual_navigation_snapshot_ids.retain(|index, id| {
            self.visual_list_blocks
                .get(*index)
                .is_some_and(|block| block.id == *id)
        });
        if let Some(existing) = self
            .visual_navigation_snapshots
            .get_mut(&snapshot.block_index)
            && existing.document_version == snapshot.document_version
            && self
                .visual_navigation_snapshot_ids
                .get(&snapshot.block_index)
                == Some(&block_id)
            && existing.source_selection == snapshot.source_selection
            && existing.marked_range == snapshot.marked_range
            && existing.source_island == snapshot.source_island
        {
            for line in snapshot.lines.drain(..) {
                if let Some(current) = existing.lines.iter_mut().find(|item| item.y == line.y) {
                    current.carets.extend(line.carets);
                    current
                        .carets
                        .sort_by(|left, right| left.x.to_f64().total_cmp(&right.x.to_f64()));
                    current.carets.dedup();
                } else {
                    existing.lines.push(line);
                }
            }
            existing
                .lines
                .sort_by(|left, right| left.y.to_f64().total_cmp(&right.y.to_f64()));
            return;
        }
        self.visual_navigation_snapshot_ids
            .insert(snapshot.block_index, block_id);
        self.visual_navigation_snapshots
            .insert(snapshot.block_index, snapshot);
    }

    pub(super) fn scroll_editor_to_offset(&mut self, offset: usize) {
        let offset = clamp_to_text_boundary(self.document.text(), offset);
        let line = self.document.text()[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        let line_height = f32::from(self.line_height);
        self.editor_scroll
            .set_offset(point(px(0.), -px(line as f32 * line_height)));
        self.sync_scroll_state.mark_driver(PaneScrollTarget::Editor);
    }

    pub(super) fn scroll_editor_typewriter_to_offset(&mut self, offset: usize) {
        let offset = clamp_to_text_boundary(self.document.text(), offset);
        let line = self.document.text()[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        // Keep the caret ~10 lines below the viewport top ("typewriter" band).
        let line_height = f32::from(self.line_height);
        let y = (line as f32 * line_height - 10. * line_height).max(0.);
        self.editor_scroll.set_offset(point(px(0.), -px(y)));
        self.sync_scroll_state.mark_driver(PaneScrollTarget::Editor);
    }

    pub(super) fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            document: self.document.clone(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
        }
    }

    pub(super) fn push_undo_snapshot(&mut self) {
        self.finish_undo_capture();
        let snapshot = self.snapshot();
        push_history_entry(&mut self.undo_stack, UndoEntry::Full(snapshot));
        self.redo_stack.clear();
    }

    pub(super) fn commit_undo_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.finish_undo_capture();
        push_history_entry(&mut self.undo_stack, UndoEntry::Full(snapshot));
        self.redo_stack.clear();
    }

    pub(super) fn finish_undo_capture(&mut self) {
        self.undo_capture = None;
        self.pending_text_edit_intent = None;
    }

    pub(super) fn prepare_undo_capture(
        &mut self,
        kind: UndoCaptureKind,
        range: &Range<usize>,
        replacement: &str,
        now: Instant,
    ) {
        let compatible = self.undo_capture.is_some_and(|capture| {
            capture.kind == kind
                && !matches!(kind, UndoCaptureKind::Atomic)
                && now.saturating_duration_since(capture.last_edit_at) <= SEMANTIC_UNDO_TIMEOUT
                && match kind {
                    UndoCaptureKind::Insert => {
                        range.is_empty() && range.start == capture.next_cursor
                    }
                    UndoCaptureKind::Delete => {
                        replacement.is_empty()
                            && (range.start == capture.next_cursor
                                || range.end == capture.next_cursor)
                    }
                    UndoCaptureKind::Ime => true,
                    UndoCaptureKind::Atomic => false,
                }
        });
        if !compatible {
            self.finish_undo_capture();
            let snapshot = self.snapshot();
            push_history_entry(&mut self.undo_stack, UndoEntry::Full(snapshot));
            self.redo_stack.clear();
        }
        if matches!(kind, UndoCaptureKind::Atomic) {
            self.undo_capture = None;
        } else {
            self.undo_capture = Some(UndoCapture {
                kind,
                last_edit_at: now,
                next_cursor: range.start + replacement.len(),
            });
        }
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
        self.finish_undo_capture();
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
        self.finish_undo_capture();
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

    pub(super) fn source_layout_is_current(&self) -> bool {
        self.source_layout_key
            .is_some_and(|key| key.version == self.document.version())
            && !self.last_lines.is_empty()
            && self.last_lines.len() == self.line_offsets.len()
            && self.line_tops.len() == self.last_lines.len() + 1
    }

    /// Convert a source byte offset to a Y coordinate in the editor's
    /// scrollable content space. The returned coordinate is independent of the
    /// viewport's current scroll offset.
    pub(super) fn source_content_y_for_offset(&self, offset: usize) -> Option<f32> {
        if !self.source_layout_is_current() {
            return None;
        }
        let text = self.document.text();
        let clamped = clamp_to_text_boundary(text, offset.min(text.len()));
        let line_index = self
            .line_offsets
            .partition_point(|start| *start <= clamped)
            .saturating_sub(1)
            .min(self.last_lines.len().saturating_sub(1));
        let line_start = self.line_offsets[line_index];
        let local_offset = clamped.saturating_sub(line_start);
        let line_top = f32::from(self.line_tops[line_index]);
        let line_height = f32::from(
            self.line_heights
                .get(line_index)
                .copied()
                .unwrap_or(self.line_height),
        );
        let local_y = self.last_lines[line_index]
            .position_for_index(local_offset, self.line_height)
            .map(|position| f32::from(position.y))
            .unwrap_or(line_height);
        Some((line_top + local_y).clamp(0., f32::from(*self.line_tops.last()?)))
    }

    /// Convert a Y coordinate in the editor's scrollable content space to the
    /// closest valid source byte offset at the left edge of that wrapped visual
    /// line.
    pub(super) fn source_offset_for_content_y(&self, content_y: f32) -> Option<usize> {
        if !self.source_layout_is_current() {
            return None;
        }
        let total = f32::from(*self.line_tops.last()?);
        let y = content_y.clamp(0., total);
        if y <= 0. {
            return Some(0);
        }
        if y >= total {
            return Some(self.document.text().len());
        }
        let line_index = self
            .line_tops
            .partition_point(|top| f32::from(*top) <= y)
            .saturating_sub(1)
            .min(self.last_lines.len().saturating_sub(1));
        let local_y = y - f32::from(self.line_tops[line_index]);
        let local_point = point(px(0.), px(local_y));
        let local_offset = match self.last_lines[line_index]
            .closest_index_for_position(local_point, self.line_height)
        {
            Ok(offset) | Err(offset) => offset,
        };
        let offset = self.line_offsets[line_index]
            .saturating_add(local_offset)
            .min(self.document.text().len());
        Some(clamp_to_text_boundary(self.document.text(), offset))
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

    pub(super) fn offset_from_utf16(&self, offset: usize) -> Option<usize> {
        utf16_offset_to_byte_offset(self.document.text(), offset)
    }

    pub(super) fn offset_to_utf16(&self, offset: usize) -> usize {
        byte_offset_to_utf16_offset(self.document.text(), offset)
    }

    pub(super) fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub(super) fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Option<Range<usize>> {
        if range_utf16.start > range_utf16.end {
            return None;
        }
        let start = self.offset_from_utf16(range_utf16.start)?;
        let end = self.offset_from_utf16(range_utf16.end)?;
        self.checked_source_range(&(start..end))
    }

    pub(super) fn relative_range_from_utf16(
        text: &str,
        range_utf16: &Range<usize>,
    ) -> Option<Range<usize>> {
        if range_utf16.start > range_utf16.end {
            return None;
        }
        let start = utf16_offset_to_byte_offset(text, range_utf16.start)?;
        let end = utf16_offset_to_byte_offset(text, range_utf16.end)?;
        text.get(start..end).map(|_| start..end)
    }

    pub(super) fn checked_source_range(&self, range: &Range<usize>) -> Option<Range<usize>> {
        self.document
            .text()
            .get(range.clone())
            .map(|_| range.clone())
    }

    pub(super) fn safe_selected_range(&self) -> Range<usize> {
        self.checked_source_range(&self.selected_range)
            .unwrap_or_else(|| {
                let caret = clamp_to_text_boundary(self.document.text(), self.selected_range.end);
                caret..caret
            })
    }
}

fn next_recovery_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
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

/// Toggles a file-tree folder between collapsed and expanded with strict
/// one-level semantics.
///
/// When expanding a previously-collapsed folder, every descendant directory
/// is recorded as collapsed so only the folder's immediate children become
/// visible — each click drills down exactly one further level instead of
/// opening the whole subtree. When collapsing a previously-expanded folder,
/// the folder itself is recorded as collapsed and the depth-based visibility
/// filter hides its entire subtree.
pub(super) fn toggle_tree_folder(
    folder: &Path,
    tree: &FileTree,
    collapsed_paths: &mut HashSet<PathBuf>,
) {
    if collapsed_paths.remove(folder) {
        // Was collapsed → expanding: reveal only immediate children by
        // collapsing every descendant directory.
        for entry in &tree.entries {
            if entry.kind == FileTreeEntryKind::Directory
                && entry.path != folder
                && entry.path.starts_with(folder)
            {
                collapsed_paths.insert(entry.path.clone());
            }
        }
    } else {
        // Was expanded → collapsing: the depth filter hides the subtree.
        collapsed_paths.insert(folder.to_path_buf());
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

pub(super) fn should_restore_session(intent: &StartupOpenIntent) -> bool {
    matches!(intent, StartupOpenIntent::None)
}

/// Filter a loaded session down to paths that still exist and are usable.
pub(super) fn filter_restorable_session(
    session: &SessionState,
) -> (Option<PathBuf>, Vec<PathBuf>, Option<PathBuf>) {
    let workspace_root = session
        .workspace_root
        .as_ref()
        .filter(|root| root.is_dir())
        .cloned();
    let open_files: Vec<PathBuf> = session
        .open_files
        .iter()
        .filter(|path| path.is_file() && is_markdown_path(path))
        .cloned()
        .collect();
    let active_file = session
        .active_file
        .as_ref()
        .filter(|path| open_files.iter().any(|open| open == *path))
        .cloned();
    (workspace_root, open_files, active_file)
}

/// Collect path-backed open tabs for the session snapshot. Untitled tabs are omitted.
pub(super) fn session_open_files_from_paths<'a>(
    paths: impl IntoIterator<Item = Option<&'a Path>>,
) -> Vec<PathBuf> {
    paths
        .into_iter()
        .flatten()
        .map(comparable_document_path)
        .collect()
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
    tabs.iter()
        .position(|tab| tab.focus_identity().as_ref() == Some(&target))
}
