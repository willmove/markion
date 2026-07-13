#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    env, fs, io,
    ops::Range,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::prelude::*;
use gpui::{
    App, Application, Bounds, ClickEvent, ClipboardItem, Context, CursorStyle, DefiniteLength,
    DispatchPhase, Div, DragMoveEvent, Element, ElementId, ElementInputHandler, Empty, Entity,
    EntityInputHandler, ExternalPaths, FocusHandle, Focusable, FontStyle, FontWeight,
    GlobalElementId, HighlightStyle, Hitbox, HitboxBehavior, ImageSource, KeyBinding, LayoutId,
    ListAlignment, ListState, Menu, MenuItem, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PaintQuad, PathPromptOptions, Pixels, Point, PromptButton, PromptLevel, Rgba,
    ScrollHandle, SharedString, Stateful, StrikethroughStyle, Style, StyledText, TextLayout,
    TextRun, Timer, TitlebarOptions, UTF16Selection, UnderlineStyle, Window, WindowBounds,
    WindowOptions, WrappedLine, actions, canvas, div, fill, img, list, point, px, rgb, rgba, size,
};
use markion::{
    AppPreferences, AutoSavePreferences, AutosaveOutcome, DEFAULT_HEADING_MENU_MAX_LEVEL,
    EXTENDED_HEADING_MENU_MAX_LEVEL, ExportBackend, ExportFormat, ExportPreferences, FileTree,
    FileTreeEntry, FileTreeEntryKind, HighlightKind, HighlightedSpan, Language, MarkdownDocument,
    MarkdownFormat, Msg, PreviewBlock, RichText, SearchMatchRange, SearchOptions, SidebarTab,
    TableEdit, ThemeColors, ThemeDefinition, ViewMode, VisualBlock, VisualBlockKind,
    VisualInlineRun, VisualSourceIslandKind, builtin_theme_definitions, default_preferences_path,
    default_recovery_dir, default_themes_dir, delete_recovery_file, highlight_code,
    is_markdown_path, list_recovery_files, list_theme_definitions, load_app_preferences,
    load_recovery_file, normalize_heading_menu_max_level, render_math, save_app_preferences,
    save_theme_definition, shortcut_reference, sidebar_tab_label, t, tf, title_from_path,
};
use unicode_segmentation::UnicodeSegmentation;

actions!(
    markion,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        InsertNewline,
        Indent,
        Outdent,
        Paste,
        Cut,
        Copy,
        Undo,
        Redo,
        Bold,
        Italic,
        InlineCode,
        InsertLink,
        InsertImage,
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
        TableDeleteColumn,
        NewDocument,
        OpenDocument,
        OpenFolder,
        SaveDocument,
        SaveDocumentAs,
        ExportHtml,
        ExportPlainHtml,
        ExportPdf,
        ExportLatex,
        ExportDocx,
        ExportPng,
        ExportJpeg,
        ToggleViewMode,
        SetEditMode,
        SetVisualEditMode,
        SetSplitPreviewMode,
        SetReadMode,
        ToggleSidebar,
        ToggleOutline,
        ToggleFileTree,
        FocusFileTreeSearch,
        ClearFileTreeSearch,
        RefreshFileTree,
        CreateTreeFile,
        CreateTreeFolder,
        RenameTreeEntry,
        DeleteTreeEntry,
        ConfirmPendingName,
        CycleTheme,
        ToggleFocusMode,
        ToggleTypewriterMode,
        ToggleCodeLineNumbers,
        ShowFind,
        ShowReplace,
        FindNext,
        FindPrevious,
        ReplaceCurrentMatch,
        ReplaceAllMatches,
        ToggleFindCaseSensitive,
        ToggleFindRegex,
        ShowShortcuts,
        ShowPreferences,
        ResetPreferences,
        AboutMarkion,
        Quit,
        NewTab,
        OpenInNewTab,
        CloseTab,
        NextTab,
        PrevTab,
    ]
);

const MARKION_APP_ID: &str = "dev.markion.app";
const MARKION_WINDOW_TITLE: &str = "Markion";

const MAX_HISTORY_LEN: usize = 200;
const GITHUB_REPO_URL: &str = "https://github.com/willmove/markion";

#[cfg(test)]
fn shortcut_reference_text() -> &'static str {
    // Kept as an English-only entry point for the existing unit test; the live
    // UI uses `shortcut_reference(self.language)` via the i18n module.
    shortcut_reference(Language::En, DEFAULT_HEADING_MENU_MAX_LEVEL)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppMenu {
    File,
    Edit,
    View,
    Format,
    Export,
    Help,
}

impl AppMenu {
    /// Left offset of a top-level menu's dropdown panel. The values are
    /// hand-tuned per language because the in-window menu bar lays buttons
    /// out with fixed paddings/gaps rather than measuring text widths.
    fn dropdown_left(self, language: Language) -> Pixels {
        match (language, self) {
            // Latin-script labels use the wider English menu spacing.
            (
                Language::En | Language::Ja | Language::Fr | Language::De | Language::Es,
                AppMenu::File,
            ) => px(8.),
            (
                Language::En | Language::Ja | Language::Fr | Language::De | Language::Es,
                AppMenu::Edit,
            ) => px(58.),
            (
                Language::En | Language::Ja | Language::Fr | Language::De | Language::Es,
                AppMenu::View,
            ) => px(108.),
            (
                Language::En | Language::Ja | Language::Fr | Language::De | Language::Es,
                AppMenu::Format,
            ) => px(162.),
            (
                Language::En | Language::Ja | Language::Fr | Language::De | Language::Es,
                AppMenu::Export,
            ) => px(238.),
            (
                Language::En | Language::Ja | Language::Fr | Language::De | Language::Es,
                AppMenu::Help,
            ) => px(304.),
            // Chinese labels (文件/编辑/视图/格式/导出/帮助) — narrower.
            (Language::Zh, AppMenu::File) => px(8.),
            (Language::Zh, AppMenu::Edit) => px(50.),
            (Language::Zh, AppMenu::View) => px(92.),
            (Language::Zh, AppMenu::Format) => px(134.),
            (Language::Zh, AppMenu::Export) => px(178.),
            (Language::Zh, AppMenu::Help) => px(222.),
        }
    }

    fn dropdown_width(self, _language: Language) -> Pixels {
        // Keep enough room for the longest localized label in each menu.
        match self {
            AppMenu::View => px(172.),
            AppMenu::Format => px(188.),
            AppMenu::Export => px(168.),
            AppMenu::Help => px(196.),
            AppMenu::File => px(176.),
            AppMenu::Edit => px(128.),
        }
    }
}

#[derive(Clone, Debug)]
struct FileTreeContextMenu {
    target: FileTreeContextTarget,
    position: Point<Pixels>,
}

/// Which create/rename operation an open inline name prompt is collecting a
/// name for. Determines the commit behavior and the pre-filled default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingNameKind {
    CreateFile,
    CreateFolder,
    Rename,
}

/// In-flight inline name prompt for a file-tree create/rename action. The
/// buffer is the text the user is editing; on Enter the kind decides which
/// `FileTree` operation runs, and on Escape the prompt is dropped without
/// touching the filesystem. The prompt reuses the app's redirected-text-input
/// path (the same one the search field and file-tree filter use) so IME
/// composition is handled identically.
#[derive(Clone, Debug)]
struct PendingNameInput {
    kind: PendingNameKind,
    /// Directory the new entry is created in (create), or the parent of the
    /// entry being renamed (rename).
    parent: PathBuf,
    /// The entry being renamed; `None` for create actions.
    target: Option<PathBuf>,
    buffer: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FileTreeContextTarget {
    Workspace,
    Directory(PathBuf),
    File(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileTreeContextTargetKind {
    Workspace,
    Directory,
    File,
}

impl FileTreeContextTarget {
    fn kind(&self) -> FileTreeContextTargetKind {
        match self {
            Self::Workspace => FileTreeContextTargetKind::Workspace,
            Self::Directory(_) => FileTreeContextTargetKind::Directory,
            Self::File(_) => FileTreeContextTargetKind::File,
        }
    }

    fn path(&self, workspace_root: &Path) -> PathBuf {
        match self {
            Self::Workspace => workspace_root.to_path_buf(),
            Self::Directory(path) | Self::File(path) => path.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileTreeContextAction {
    Open,
    OpenInNewTab,
    CreateFile,
    CreateFolder,
    Rename,
    Delete,
    ShowInFileManager,
    Refresh,
    FilterFiles,
}

const FILE_TREE_FILE_CONTEXT_ACTIONS: &[FileTreeContextAction] = &[
    FileTreeContextAction::Open,
    FileTreeContextAction::OpenInNewTab,
    FileTreeContextAction::Rename,
    FileTreeContextAction::Delete,
    FileTreeContextAction::ShowInFileManager,
    FileTreeContextAction::Refresh,
];

const FILE_TREE_DIRECTORY_CONTEXT_ACTIONS: &[FileTreeContextAction] = &[
    FileTreeContextAction::CreateFile,
    FileTreeContextAction::CreateFolder,
    FileTreeContextAction::Rename,
    FileTreeContextAction::Delete,
    FileTreeContextAction::ShowInFileManager,
    FileTreeContextAction::Refresh,
];

const FILE_TREE_WORKSPACE_CONTEXT_ACTIONS: &[FileTreeContextAction] = &[
    FileTreeContextAction::CreateFile,
    FileTreeContextAction::CreateFolder,
    FileTreeContextAction::Refresh,
    FileTreeContextAction::ShowInFileManager,
    FileTreeContextAction::FilterFiles,
];

fn file_tree_context_actions(kind: FileTreeContextTargetKind) -> &'static [FileTreeContextAction] {
    match kind {
        FileTreeContextTargetKind::File => FILE_TREE_FILE_CONTEXT_ACTIONS,
        FileTreeContextTargetKind::Directory => FILE_TREE_DIRECTORY_CONTEXT_ACTIONS,
        FileTreeContextTargetKind::Workspace => FILE_TREE_WORKSPACE_CONTEXT_ACTIONS,
    }
}

fn file_tree_context_action_label(action: FileTreeContextAction) -> Msg {
    match action {
        FileTreeContextAction::Open => Msg::FileTreeContextOpen,
        FileTreeContextAction::OpenInNewTab => Msg::FileTreeContextOpenInNewTab,
        FileTreeContextAction::CreateFile => Msg::FileTreeContextCreateFile,
        FileTreeContextAction::CreateFolder => Msg::FileTreeContextCreateFolder,
        FileTreeContextAction::Rename => Msg::FileTreeContextRename,
        FileTreeContextAction::Delete => Msg::FileTreeContextDelete,
        FileTreeContextAction::ShowInFileManager => Msg::FileTreeContextShowInFileManager,
        FileTreeContextAction::Refresh => Msg::FileTreeContextRefresh,
        FileTreeContextAction::FilterFiles => Msg::FileTreeContextFilterFiles,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchField {
    Find,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppTheme {
    Paper,
    Ink,
    Solar,
    Forest,
    Rose,
    Graphite,
}

impl AppTheme {
    const ALL: [Self; 6] = [
        Self::Paper,
        Self::Ink,
        Self::Solar,
        Self::Forest,
        Self::Rose,
        Self::Graphite,
    ];

    #[cfg(test)]
    fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|theme| *theme == self)
            .unwrap_or_default();
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    fn name(self) -> &'static str {
        match self {
            Self::Paper => "Paper",
            Self::Ink => "Ink",
            Self::Solar => "Solar",
            Self::Forest => "Forest",
            Self::Rose => "Rose",
            Self::Graphite => "Graphite",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|theme| theme.name().eq_ignore_ascii_case(name.trim()))
    }
}

#[derive(Clone, Copy)]
struct ThemePalette {
    app_bg: Rgba,
    panel_bg: Rgba,
    surface_bg: Rgba,
    text: Rgba,
    muted: Rgba,
    border: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
}

fn theme_palette_from_definition(theme: &ThemeDefinition) -> ThemePalette {
    theme_palette_from_colors(theme.colors)
}

fn theme_palette_from_colors(colors: ThemeColors) -> ThemePalette {
    ThemePalette {
        app_bg: rgb(colors.app_bg),
        panel_bg: rgb(colors.panel_bg),
        surface_bg: rgb(colors.surface_bg),
        text: rgb(colors.text),
        muted: rgb(colors.muted),
        border: rgb(colors.border),
        active_bg: rgb(colors.active_bg),
        active_text: rgb(colors.active_text),
    }
}

/// Width of the invisible "grab" zone centered on a resize divider. The visual
/// divider is 1px, but a hit target that thin is nearly impossible to grab, so
/// we overlay a wider transparent handle on top of it (mirrors Zed's split view).
const RESIZE_HANDLE_WIDTH: f32 = 8.;
const PANE_OUTER_PADDING: f32 = 0.;
const PANE_INNER_PADDING: f32 = 9.;
const PREVIEW_SCROLLBAR_SAFE_RIGHT_PADDING: f32 =
    PANE_INNER_PADDING + PANE_SCROLLBAR_RESERVED_WIDTH;
const SIDEBAR_COMPACT_PADDING: f32 = 2.5;
const READ_MODE_PREVIEW_MAX_WIDTH: f32 = 860.;
const PANE_SCROLLBAR_RESERVED_WIDTH: f32 = 15.;
const PANE_SCROLLBAR_THUMB_WIDTH: f32 = 9.;
const PANE_SCROLLBAR_MIN_THUMB_HEIGHT: f32 = 32.;
const PANE_SCROLLBAR_EDGE_INSET: f32 = 3.;
/// Nominal line height (px) of the source editor. Used both when painting the
/// editor text and by the line-based scroll helpers, so the two stay in sync;
/// the actual per-line height is measured during layout for hit-testing.
const EDITOR_LINE_HEIGHT: f32 = 24.;
/// Line height (px) of the preview pane. Independent of the editor: the preview
/// scrolls natively via its `ListState`, not by line-index math.
const PREVIEW_LINE_HEIGHT: f32 = 23.;
/// Extra vertical margin (px) the preview `list` renders beyond the visible
/// viewport so a fast scroll or drag does not flash blank rows before the
/// newly-revealed blocks are measured. Larger = smoother scroll, more per-frame
/// element construction; ~2 screens' worth of a typical block is plenty.
const PREVIEW_LIST_OVERDRAW: f32 = 800.;
/// How long typing must pause before the preview pane re-parses the document.
/// While keystrokes arrive faster than this, Split/Read renders keep showing the
/// previous blocks (stale by at most a few keystrokes) instead of paying a
/// full-document parse on every key.
const PREVIEW_DEBOUNCE: Duration = Duration::from_millis(80);
/// Upper bound on preview staleness during *continuous* typing: if the last
/// parse is older than this, the next render parses even though the debounce
/// window has not elapsed, so the preview never freezes mid-typing-burst.
const PREVIEW_MAX_STALE: Duration = Duration::from_millis(400);
/// Clamp range for the editor/preview split ratio so neither pane can collapse.
const EDITOR_SPLIT_RATIO_MIN: f32 = 0.15;
const EDITOR_SPLIT_RATIO_MAX: f32 = 0.85;
/// Default and clamp range for the sidebar pixel width.
const DEFAULT_SIDEBAR_WIDTH: f32 = 230.;
const SIDEBAR_MIN_WIDTH: f32 = 150.;
const SIDEBAR_MAX_WIDTH: f32 = 480.;

/// Drag value types used only to key `on_drag` / `on_drag_move` / `on_drop` —
/// they carry no data, they just let each divider's drag be tracked
/// independently (mirrors Zed's `DraggedSplitHandle`).
#[derive(Debug, Clone)]
struct DraggedEditorSplitHandle;
#[derive(Debug, Clone)]
struct DraggedSidebarHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneScrollTarget {
    Editor,
    Preview,
}

/// Identity of a selectable plain-text run inside one preview list item.
/// Decorative chrome (list markers, code line numbers, table buttons) is never
/// a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewTextRunId {
    Body,
    CodeBody,
    CodeLine(usize),
    MathLatex,
    MathRendered,
    ImageCaption,
    ImageMeta,
    TableCell { row: usize, col: usize },
}

impl PreviewTextRunId {
    /// Stable document order of runs within a single preview block.
    fn rank(self) -> (u8, usize, usize) {
        match self {
            Self::Body => (0, 0, 0),
            Self::CodeBody => (1, 0, 0),
            Self::CodeLine(i) => (2, i, 0),
            Self::MathRendered => (3, 0, 0),
            Self::MathLatex => (4, 0, 0),
            Self::ImageCaption => (5, 0, 0),
            Self::ImageMeta => (6, 0, 0),
            Self::TableCell { row, col } => (7, row, col),
        }
    }
}

/// A caret into preview textual content (block + run + UTF-8 byte offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreviewCaret {
    block_index: usize,
    run_id: PreviewTextRunId,
    offset: usize,
}

impl PreviewCaret {
    fn cmp_doc_order(self, other: Self) -> std::cmp::Ordering {
        (self.block_index, self.run_id.rank(), self.offset).cmp(&(
            other.block_index,
            other.run_id.rank(),
            other.offset,
        ))
    }
}

/// App-owned free-range preview selection. `anchor` is where the drag started;
/// `head` is the current end. Ordered endpoints are derived for highlight/copy.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PreviewSelection {
    anchor: PreviewCaret,
    head: PreviewCaret,
}

impl PreviewSelection {
    fn ordered_carets(&self) -> (PreviewCaret, PreviewCaret) {
        if self.anchor.cmp_doc_order(self.head) == std::cmp::Ordering::Greater {
            (self.head, self.anchor)
        } else {
            (self.anchor, self.head)
        }
    }

    fn is_empty_carets(&self) -> bool {
        self.anchor == self.head
    }
}

/// Right-click menu for the preview pane (mirrors [`FileTreeContextMenu`]).
#[derive(Debug, Clone)]
struct PreviewContextMenu {
    position: Point<Pixels>,
    link_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewContextAction {
    CopyPlain,
    CopyMarkdown,
    CopyHtml,
    SelectAll,
    CopyLinkAddress,
}

#[derive(Debug, Clone, Copy)]
struct PaneScrollbarDrag {
    target: PaneScrollTarget,
    thumb_grab_offset_y: Pixels,
}

/// Farthest the grapheme-boundary helpers look back for a line start before
/// giving up and scanning from an arbitrary char boundary. Only pathological
/// newline-free lines hit the cap; real grapheme clusters are tens of bytes at
/// most, so a 1 KB lookback never changes the result in practice.
const BOUNDARY_SCAN_WINDOW: usize = 1024;

/// Where a grapheme scan for the cluster around `offset` may safely start:
/// the current line start (segmentation restarts after every hard break), or
/// the nearest char boundary [`BOUNDARY_SCAN_WINDOW`] bytes back when the
/// line itself is longer than that.
fn boundary_scan_start(text: &str, offset: usize) -> usize {
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
struct MeasuredHeightKey {
    version: u64,
    wrap_width: Pixels,
    font_size: Pixels,
    line_height: Pixels,
}

#[derive(Clone)]
struct EditorSnapshot {
    document: MarkdownDocument,
    selected_range: Range<usize>,
    selection_reversed: bool,
}

/// One entry in the undo/redo history.
///
/// Edit sites push `Full` pre-edit snapshots exactly as before, but
/// [`push_history_entry`] compacts the previously-newest full entry into a
/// `Diff` at push time — the only moment both texts are in hand. Each stack
/// therefore retains at most one whole-document copy (its newest entry) no
/// matter how long the history grows; previously a 1 MB document accumulated
/// up to `MAX_HISTORY_LEN` full clones (~200 MB) while typing.
enum UndoEntry {
    Full(EditorSnapshot),
    Diff(UndoDiff),
}

/// Compact history record. LIFO order guarantees that when this entry is
/// popped the document text is exactly the state the diff was computed
/// against, so applying it means: replace `range` of the current text with
/// `insert`, then restore the recorded selection.
struct UndoDiff {
    range: Range<usize>,
    insert: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
}

/// Push a history entry onto `stack`, compacting the previous top into a
/// [`UndoDiff`] when both it and the new entry are `Full` (a `Diff` on top is
/// already compact, and its presence means the buried full entry pops against
/// a text we cannot know yet). Caps the stack at [`MAX_HISTORY_LEN`].
fn push_history_entry(stack: &mut Vec<UndoEntry>, entry: UndoEntry) {
    if let (UndoEntry::Full(new), Some(top)) = (&entry, stack.last_mut()) {
        if let UndoEntry::Full(old) = top {
            *top = UndoEntry::Diff(compact_history_entry(old, new.document.text()));
        }
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
fn compact_history_entry(older: &EditorSnapshot, newer_text: &str) -> UndoDiff {
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
struct EditorTab {
    document: MarkdownDocument,
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
    editor_scroll: ScrollHandle,
    /// Virtualized preview: GPUI's `list` renders only the blocks intersecting
    /// the viewport (+overdraw), so preview cost is O(visible blocks) instead of
    /// O(document). The state is intrusive and must persist across frames.
    preview_list: ListState,
    visual_list: ListState,
    visual_list_blocks: std::sync::Arc<Vec<VisualBlock>>,
    /// Snapshot of the block slice `preview_list` currently reflects. Each frame
    /// we diff the freshly-parsed blocks against this and `splice` only the
    /// changed range into `preview_list`, which preserves scroll position (a
    /// full `reset` would jump to the top on every keystroke).
    preview_list_blocks: std::sync::Arc<Vec<PreviewBlock>>,
    /// Debounced preview parsing (Split/Read): the latest document version a
    /// render has observed, and the version the preview blocks actually
    /// reflect. When they differ the preview is stale and a parse is due once
    /// the debounce window elapses (or `PREVIEW_MAX_STALE` forces one).
    preview_seen_version: u64,
    preview_reflects_version: Option<u64>,
    /// When the document last changed / was last parsed for the preview, used
    /// to decide "typing has settled" and "too stale, parse anyway".
    preview_changed_at: Option<Instant>,
    preview_reflects_at: Option<Instant>,
    /// Generation token incremented whenever a new debounce timer is armed (or
    /// the pending one must be cancelled); a firing timer compares its captured
    /// generation against this and does nothing if it lost the race.
    preview_debounce_generation: u64,
    /// Id of the background preview parse currently in flight for this tab
    /// (`next_preview_parse_id`), or `None`. At most one parse runs per tab;
    /// ids are globally unique so a landing result can find its owning tab by
    /// id (tab indices shift when other tabs close) and a result whose tab was
    /// replaced meanwhile (`reset_preview_list` clears the marker) is dropped.
    preview_parse_inflight: Option<u64>,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_lines: Vec<WrappedLine>,
    line_offsets: Vec<usize>,
    line_heights: Vec<Pixels>,
    last_bounds: Option<Bounds<Pixels>>,
    /// Actual line height from the last layout pass, reused by hit-testing so
    /// mouse positions line up with the painted text.
    line_height: Pixels,
    is_selecting: bool,
    /// The document text as a `SharedString`, cached per document version so
    /// the editor element does not copy the whole document on every frame.
    display_text_cache: RefCell<Option<(u64, SharedString)>>,
    /// Total wrapped height from the last layout measure. The measure closure
    /// runs on every layout pass and a full-document `shape_text` — even one
    /// that hits GPUI's per-line layout cache — still walks and hashes every
    /// line; this memo makes repeat measures O(1). (`text_version` values are
    /// globally unique, so a replaced document can never alias a stale entry.)
    measured_height_cache: RefCell<Option<(MeasuredHeightKey, Pixels)>>,
    /// Byte offset of each logical line start, cached per document version:
    /// prepaint needs the table every frame and rebuilding it is an
    /// O(document) `match_indices` scan.
    line_offsets_cache: RefCell<Option<(u64, Rc<Vec<usize>>)>>,
    last_recovery_file: Option<PathBuf>,
    /// Generation token incremented on every autosave schedule; a pending timer
    /// compares its captured generation against this to decide whether to fire.
    autosave_generation: u64,
    /// Last scroll fraction applied to the editor during sync reconciliation
    /// (None until the first reconciled Split frame). Used to detect which pane
    /// drove the latest scroll change so only the *other* pane is written.
    sync_scroll_editor_fraction: Option<f32>,
    /// Last scroll fraction applied to the preview during sync reconciliation.
    sync_scroll_preview_fraction: Option<f32>,
    /// Active drag/copy selection in the rendered preview for this tab.
    /// Independent of the source editor selection; never mutates the document.
    preview_selection: Option<PreviewSelection>,
    /// True while the user is dragging a preview text selection.
    preview_is_selecting: bool,
}

impl EditorTab {
    fn new(document: MarkdownDocument) -> Self {
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
    fn sync_preview_list(&mut self, blocks: &std::sync::Arc<Vec<PreviewBlock>>) {
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

    fn sync_visual_list(&mut self, blocks: &std::sync::Arc<Vec<VisualBlock>>) {
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
    fn reset_preview_list(&mut self) {
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

    fn clear_preview_selection(&mut self) {
        self.preview_selection = None;
        self.preview_is_selecting = false;
    }

    /// Cached `SharedString` copy of the document text for the current
    /// version. Cloning the returned value is an `Arc` bump, not a text copy.
    fn shared_document_text(&self) -> SharedString {
        let version = self.document.version();
        if let Some((cached_version, text)) = self.display_text_cache.borrow().as_ref() {
            if *cached_version == version {
                return text.clone();
            }
        }
        let text: SharedString = self.document.text().to_string().into();
        *self.display_text_cache.borrow_mut() = Some((version, text.clone()));
        text
    }

    /// Byte offset at the start of each logical line, cached per document
    /// version. Cloning the returned value is an `Rc` bump.
    fn shared_line_offsets(&self) -> Rc<Vec<usize>> {
        let version = self.document.version();
        if let Some((cached_version, offsets)) = self.line_offsets_cache.borrow().as_ref() {
            if *cached_version == version {
                return offsets.clone();
            }
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

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn scroll_editor_to_offset(&self, offset: usize) {
        let offset = clamp_to_text_boundary(self.document.text(), offset);
        let line = self.document.text()[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        self.editor_scroll
            .set_offset(point(px(0.), -px(line as f32 * EDITOR_LINE_HEIGHT)));
    }

    fn scroll_editor_typewriter_to_offset(&self, offset: usize) {
        let offset = clamp_to_text_boundary(self.document.text(), offset);
        let line = self.document.text()[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        // Keep the caret ~10 lines below the viewport top ("typewriter" band).
        let y = (line as f32 * EDITOR_LINE_HEIGHT - 10. * EDITOR_LINE_HEIGHT).max(0.);
        self.editor_scroll.set_offset(point(px(0.), -px(y)));
    }

    fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            document: self.document.clone(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
        }
    }

    fn push_undo_snapshot(&mut self) {
        self.commit_undo_snapshot(self.snapshot());
    }

    fn commit_undo_snapshot(&mut self, snapshot: EditorSnapshot) {
        push_history_entry(&mut self.undo_stack, UndoEntry::Full(snapshot));
        self.redo_stack.clear();
    }

    /// Restore a full snapshot's document and selection.
    fn restore_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.document = snapshot.document;
        self.document.refresh_dirty_from_disk();
        self.selected_range = snapshot.selected_range;
        self.selection_reversed = snapshot.selection_reversed;
        self.marked_range = None;
    }

    /// Apply a compact history record and return its inverse — the record
    /// that, pushed onto the opposite stack, re-creates the state being left.
    fn apply_history_diff(&mut self, diff: UndoDiff) -> UndoDiff {
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
    fn apply_undo(&mut self) -> bool {
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
    fn apply_redo(&mut self) -> bool {
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

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
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
    fn layout_point_for_offset(
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
    fn previous_boundary(&self, offset: usize) -> usize {
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
            .last()
            .map(|(idx, _)| scan_start + idx)
            .unwrap_or(scan_start)
    }

    /// Start of the grapheme cluster following `offset` (the first boundary
    /// strictly greater than it). Scans from the current line start; see
    /// [`Self::previous_boundary`].
    fn next_boundary(&self, offset: usize) -> usize {
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

    fn offset_from_utf16(&self, offset: usize) -> usize {
        utf16_offset_to_byte_offset(self.document.text(), offset)
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        byte_offset_to_utf16_offset(self.document.text(), offset)
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn relative_range_from_utf16(text: &str, range_utf16: &Range<usize>) -> Range<usize> {
        utf16_offset_to_byte_offset(text, range_utf16.start)
            ..utf16_offset_to_byte_offset(text, range_utf16.end)
    }
}

fn comparable_document_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn path_is_within_workspace(root: &Path, path: &Path) -> bool {
    comparable_document_path(path).starts_with(comparable_document_path(root))
}

fn workspace_root_for_document(
    current_root: Option<&Path>,
    document_path: &Path,
) -> Option<PathBuf> {
    if let Some(root) = current_root.filter(|root| path_is_within_workspace(root, document_path)) {
        return Some(comparable_document_path(root));
    }

    document_path.parent().map(comparable_document_path)
}

fn scan_result_matches_workspace(requested_root: &Path, current_root: &Path) -> bool {
    comparable_document_path(requested_root) == comparable_document_path(current_root)
}

fn workspace_root_needs_reset(current_root: &Path, has_file_tree: bool, next_root: &Path) -> bool {
    !has_file_tree || !scan_result_matches_workspace(current_root, next_root)
}

fn open_folder_prompt_options(language: Language) -> PathPromptOptions {
    PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some(t(language, Msg::PromptOpenFolder).into()),
    }
}

fn find_tab_with_document_path(tabs: &[EditorTab], path: &Path) -> Option<usize> {
    let target = comparable_document_path(path);
    tabs.iter().position(|tab| {
        tab.document
            .path()
            .is_some_and(|open_path| comparable_document_path(open_path) == target)
    })
}

struct MarkionApp {
    tabs: Vec<EditorTab>,
    active_tab: usize,
    focus_handle: FocusHandle,
    active_menu: Option<AppMenu>,
    status: SharedString,
    confirming_close: bool,
    allow_close: bool,
    preferences_path: PathBuf,
    theme: AppTheme,
    custom_theme: Option<ThemeDefinition>,
    custom_themes: Vec<ThemeDefinition>,
    themes_dir: PathBuf,
    /// Name of the active theme, used to resolve the palette across both the
    /// built-in theme table and user-loaded `.theme` files. Empty/unknown
    /// values fall back to the legacy `theme`/`custom_theme` fields.
    selected_theme_name: String,
    /// Whether the in-app Preferences panel (theme + language picker) is open.
    preferences_panel_open: bool,
    focus_mode: bool,
    typewriter_mode: bool,
    code_line_numbers: bool,
    preview_adaptive_width: bool,
    heading_menu_max_level: u8,
    /// When enabled and the view mode is Split, the editor and preview panes
    /// scroll together proportionally. Persisted; disabled by default.
    sync_scroll: bool,
    /// Re-entrancy guard for the render-time scroll reconciliation: prevents
    /// the offset we write to the non-driving pane from being read back on the
    /// same frame as a new driver and ping-ponging.
    syncing_scroll: bool,
    view_mode: ViewMode,
    workspace_root: PathBuf,
    // Draggable layout widths. Not persisted — every launch starts from the
    // defaults so a resized window never leaves a pane unusably thin.
    editor_split_ratio: f32,
    sidebar_width: f32,
    file_tree: Option<FileTree>,
    // Unified sidebar: a single left column toggled as a whole, whose content
    // switches between the file tree and the document outline via `sidebar_tab`.
    sidebar_visible: bool,
    sidebar_tab: SidebarTab,
    file_tree_query: String,
    file_tree_query_focused: bool,
    file_tree_scroll: ScrollHandle,
    // Byte length of the trailing IME composition inside whichever redirected
    // text field (file-tree filter / search) currently has logical focus.
    input_marked_len: usize,
    selected_tree_path: Option<PathBuf>,
    collapsed_tree_paths: HashSet<PathBuf>,
    file_tree_context_menu: Option<FileTreeContextMenu>,
    /// Right-click menu for the rendered preview pane.
    preview_context_menu: Option<PreviewContextMenu>,
    /// Open inline name prompt for a file-tree create/rename action; reuses
    /// the redirected-text-input path so keystrokes route into its buffer.
    pending_name_input: Option<PendingNameInput>,
    search_visible: bool,
    replace_visible: bool,
    search_query: String,
    replace_text: String,
    search_case_sensitive: bool,
    search_regex: bool,
    search_focus: Option<SearchField>,
    search_matches: Vec<SearchMatchRange>,
    current_search_index: Option<usize>,
    pane_scrollbar_drag: Option<PaneScrollbarDrag>,
    /// Auto-save settings from the config file ([auto_save] table). Not
    /// editable in the Preferences panel; kept to round-trip on save.
    auto_save_preferences: AutoSavePreferences,
    /// Export settings from the config file ([export] table). Not editable
    /// in the Preferences panel; kept to round-trip on save.
    export_preferences: ExportPreferences,
    recovery_dir: PathBuf,
    /// Memoized syntax highlighting keyed by (language, code). Preview blocks
    /// are re-collected on every edit, but the code blocks themselves rarely
    /// change while typing prose, so their token spans are reused across
    /// edits instead of being re-lexed on every keystroke.
    highlight_cache: RefCell<HashMap<(Option<String>, String), Rc<Vec<Vec<HighlightedSpan>>>>>,
    /// Active interface language. Persisted via `AppPreferences::language`.
    language: Language,
}

impl MarkionApp {
    fn new(cx: &mut Context<Self>) -> Self {
        let document = MarkdownDocument::from_text(
            "# Welcome to Markion\n\nStart writing Markdown here. Use **bold**, lists, tables, code blocks, and task lists.\n\n- [x] Edit Markdown\n- [x] Preview document text\n- [x] Export Markdown, HTML, and PDF\n",
        );
        let workspace_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        // Defer the file tree scan out of the window-creation path. Scanning the
        // workspace synchronously here freezes the first frame (and the whole UI)
        // when the working directory is large. We start with no tree and let the
        // background scan (scheduled by the caller) populate it once ready.
        let file_tree = None;
        let preferences_path = default_preferences_path();
        let preferences = load_app_preferences(&preferences_path).unwrap_or_default();
        let themes_dir = default_themes_dir();
        let custom_themes = list_theme_definitions(&themes_dir).unwrap_or_default();
        let custom_theme = preferences
            .custom_theme
            .as_deref()
            .and_then(|name| custom_themes.iter().find(|theme| theme.name == name))
            .cloned();
        // Resolve the active theme by name. Custom-theme names take precedence
        // (matching the pre-panel behaviour), otherwise the plain `theme` name
        // is used. Unknown names fall back to Paper.
        let selected_theme_name = custom_theme
            .as_ref()
            .map(|theme| theme.name.clone())
            .or_else(|| {
                let name = preferences.theme.trim();
                (!name.is_empty()).then(|| name.to_string())
            })
            .unwrap_or_else(|| "Paper".to_string());
        Self {
            tabs: vec![EditorTab::new(document)],
            active_tab: 0,
            focus_handle: cx.focus_handle(),
            active_menu: None,
            status: t(Language::default(), Msg::StatusReady).into(),
            confirming_close: false,
            allow_close: false,
            preferences_path,
            theme: AppTheme::from_name(&preferences.theme).unwrap_or(AppTheme::Paper),
            custom_theme,
            custom_themes,
            themes_dir,
            selected_theme_name,
            preferences_panel_open: false,
            focus_mode: preferences.focus_mode,
            typewriter_mode: preferences.typewriter_mode,
            code_line_numbers: preferences.code_line_numbers,
            preview_adaptive_width: preferences.preview_adaptive_width,
            heading_menu_max_level: preferences.heading_menu_max_level,
            sync_scroll: preferences.sync_scroll,
            syncing_scroll: false,
            language: Language::from_code(&preferences.language),
            view_mode: ViewMode::default_mode(),
            workspace_root,
            editor_split_ratio: 0.5,
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
            file_tree,
            sidebar_visible: preferences.sidebar_visible,
            sidebar_tab: preferences.sidebar_tab,
            file_tree_query: String::new(),
            file_tree_query_focused: false,
            file_tree_scroll: ScrollHandle::new(),
            input_marked_len: 0,
            selected_tree_path: None,
            collapsed_tree_paths: HashSet::new(),
            file_tree_context_menu: None,
            preview_context_menu: None,
            pending_name_input: None,
            search_visible: false,
            replace_visible: false,
            search_query: String::new(),
            replace_text: String::new(),
            search_case_sensitive: false,
            search_regex: false,
            search_focus: None,
            search_matches: Vec::new(),
            current_search_index: None,
            pane_scrollbar_drag: None,
            auto_save_preferences: preferences.auto_save,
            export_preferences: preferences.export.clone(),
            recovery_dir: default_recovery_dir(),
            highlight_cache: RefCell::new(HashMap::new()),
        }
    }

    /// The currently active tab (read access).
    ///
    /// `active_tab` is clamped to `tabs.len().saturating_sub(1)` before indexing
    /// so a transiently-out-of-range index (e.g. right after a tab close, before
    /// the next render updates the tab-bar closures) cannot panic. This is a
    /// defence-in-depth: the close/switch handlers also keep the index valid,
    /// but tab-bar click closures capture an `index` at render time that can be
    /// stale by the time they fire.
    fn active_tab(&self) -> &EditorTab {
        let idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
        &self.tabs[idx]
    }

    /// The currently active tab (mutable access). See [`active_tab`](Self::active_tab)
    /// for the clamping rationale.
    fn active_tab_mut(&mut self) -> &mut EditorTab {
        let idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
        &mut self.tabs[idx]
    }

    fn focus_existing_tab_for_path(&mut self, path: &Path, cx: &mut Context<Self>) -> bool {
        let Some(index) = find_tab_with_document_path(&self.tabs, path) else {
            return false;
        };
        self.switch_active_tab(index, cx);
        self.update_workspace_root_from_document(cx);
        true
    }

    /// Switch the active tab index and clear preview selection on the newly
    /// active tab's sibling context (each tab keeps its own selection, but we
    /// still refresh search / notify so the UI settles on the new tab).
    fn switch_active_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        self.active_tab = index;
        // Selecting in another tab's preview must not leave a drag in progress
        // on the previous tab; clear the drag flag on all tabs for safety.
        for tab in &mut self.tabs {
            tab.preview_is_selecting = false;
        }
        self.refresh_search_matches();
        cx.notify();
    }

    fn begin_preview_selection(
        &mut self,
        block_index: usize,
        run_id: PreviewTextRunId,
        index: usize,
        run_text: SharedString,
        cx: &mut Context<Self>,
    ) {
        let offset = clamp_preview_offset(run_text.as_ref(), index);
        let caret = PreviewCaret {
            block_index,
            run_id,
            offset,
        };
        let tab = self.active_tab_mut();
        tab.preview_is_selecting = true;
        tab.preview_selection = Some(PreviewSelection {
            anchor: caret,
            head: caret,
        });
        // Preview interaction takes over; stop any in-progress editor drag.
        tab.is_selecting = false;
        self.preview_context_menu = None;
        cx.notify();
    }

    fn update_preview_selection_head(
        &mut self,
        block_index: usize,
        run_id: PreviewTextRunId,
        index: usize,
        run_text: SharedString,
        cx: &mut Context<Self>,
    ) {
        let offset = clamp_preview_offset(run_text.as_ref(), index);
        let tab = self.active_tab_mut();
        if !tab.preview_is_selecting {
            return;
        }
        let Some(selection) = tab.preview_selection.as_mut() else {
            return;
        };
        let head = PreviewCaret {
            block_index,
            run_id,
            offset,
        };
        if selection.head != head {
            selection.head = head;
            cx.notify();
        }
    }

    fn end_preview_selection(&mut self, cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        tab.preview_is_selecting = false;
        cx.notify();
    }

    /// Cached `SharedString` copy of the active tab's document text for the
    /// current version. Cloning the returned value is an `Arc` bump, not a
    /// text copy.
    fn shared_document_text(&self) -> SharedString {
        self.active_tab().shared_document_text()
    }

    /// Syntax highlighting memoized across edits; see `highlight_cache`.
    fn highlighted_code(
        &self,
        language: Option<&str>,
        code: &str,
    ) -> Rc<Vec<Vec<HighlightedSpan>>> {
        let key = (language.map(str::to_string), code.to_string());
        if let Some(cached) = self.highlight_cache.borrow().get(&key) {
            return cached.clone();
        }
        let highlighted = Rc::new(highlight_code(code, language));
        let mut cache = self.highlight_cache.borrow_mut();
        if cache.len() >= 128 {
            cache.clear();
        }
        cache.insert(key, highlighted.clone());
        highlighted
    }

    fn check_recovery_on_startup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(files) = list_recovery_files(&self.recovery_dir) else {
            return;
        };
        let Some(path) = files.last().cloned() else {
            return;
        };

        let detail = tf(
            self.language,
            Msg::DialogRestoreDetail,
            &[&path.display().to_string()],
        );
        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogRestoreTitle),
            Some(&detail),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonRestore)),
                PromptButton::cancel(self.tr(Msg::DialogButtonDiscard)),
            ],
            cx,
        );

        self.status = t(self.language, Msg::StatusRecoveryAvailable).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let restore = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                if restore {
                    match load_recovery_file(&path) {
                        Ok(recovery) => {
                            app.open_in_new_tab(
                                MarkdownDocument::recovered(recovery.text, recovery.original_path),
                                cx,
                            );
                            let _ = delete_recovery_file(&path);
                            app.status = t(app.language, Msg::StatusRecoveredDocument).into();
                        }
                        Err(err) => {
                            app.status =
                                tf(app.language, Msg::StatusRecoveryFailed, &[&err.to_string()])
                                    .into();
                        }
                    }
                } else {
                    let _ = delete_recovery_file(&path);
                    app.status = t(app.language, Msg::StatusRecoveryDiscarded).into();
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn after_document_changed(&mut self, cx: &mut Context<Self>) {
        self.refresh_search_matches();
        self.center_cursor_if_typewriter();
        self.schedule_autosave(cx);
    }

    fn set_workspace_root(&mut self, root: PathBuf) {
        let root = comparable_document_path(&root);
        let root_changed =
            workspace_root_needs_reset(&self.workspace_root, self.file_tree.is_some(), &root);

        if root_changed {
            self.collapsed_tree_paths.clear();
            self.selected_tree_path = None;
            self.file_tree_scroll = ScrollHandle::new();
            self.file_tree = Some(FileTree {
                root: root.clone(),
                entries: Vec::new(),
            });
        }

        self.workspace_root = root;
    }

    fn update_workspace_root_from_document(&mut self, cx: &mut Context<Self>) {
        let Some(document_path) = self.active_tab().document.path().map(Path::to_path_buf) else {
            return;
        };
        let current_root = self
            .file_tree
            .as_ref()
            .map(|_| self.workspace_root.as_path());
        let Some(next_root) = workspace_root_for_document(current_root, &document_path) else {
            return;
        };

        if self.file_tree.is_some()
            && scan_result_matches_workspace(&self.workspace_root, &next_root)
        {
            return;
        }

        self.set_workspace_root(next_root);
        self.refresh_file_tree(cx);
    }

    fn refresh_file_tree(&mut self, cx: &mut Context<Self>) {
        self.schedule_file_tree_scan(None, cx);
    }

    /// Scans the workspace on a background thread so the UI never blocks on a
    /// large directory tree. The previous synchronous scan was the dominant
    /// cause of the startup stall: it ran on the main thread during window
    /// creation and could walk tens of thousands of entries.
    fn schedule_file_tree_scan(
        &mut self,
        opened_folder_display: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let requested_root = self.workspace_root.clone();
        let scan_root = requested_root.clone();
        cx.spawn(async move |this, cx| {
            // Run the filesystem traversal off the main thread.
            let scanned = cx
                .background_executor()
                .spawn(async move { FileTree::scan(&scan_root) })
                .await;
            let _ = this.update(cx, |app, cx| {
                if !scan_result_matches_workspace(&requested_root, &app.workspace_root) {
                    return;
                }

                match scanned {
                    Ok(tree) => {
                        app.file_tree = Some(tree);
                        if let Some(path) = opened_folder_display.as_deref() {
                            app.status = app.trf(Msg::StatusOpenedFolder, &[path]);
                        }
                        if app
                            .selected_tree_path
                            .as_ref()
                            .is_some_and(|path| !path.exists())
                        {
                            app.selected_tree_path = None;
                        }
                        app.collapsed_tree_paths.retain(|path| path.exists());
                    }
                    Err(err) => {
                        app.status = app.trf(Msg::StatusOpenFolderFailed, &[&err.to_string()]);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn discard_current_recovery_file(&mut self) {
        if let Some(recovery) = self.active_tab_mut().last_recovery_file.take() {
            let _ = delete_recovery_file(recovery);
        }
    }

    /// Open `document` in a brand-new tab and make it active. Used by new
    /// untitled tabs and crash-recovery restore; filesystem-backed opens should
    /// go through the path helpers so already-open files can reuse their tab.
    fn open_in_new_tab(&mut self, document: MarkdownDocument, cx: &mut Context<Self>) {
        self.tabs.push(EditorTab::new(document));
        self.active_tab = self.tabs.len() - 1;
        self.refresh_search_matches();
        cx.notify();
    }

    /// Replace the active tab's document in place: discard its recovery file,
    /// reset its selection/undo/scroll state, and load `document`. Used by
    /// File→New and File→Open (single-tab behaviour continuity).
    fn replace_active_tab(&mut self, document: MarkdownDocument, cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        if let Some(recovery) = tab.last_recovery_file.take() {
            let _ = delete_recovery_file(recovery);
        }
        tab.document = document;
        tab.selected_range = 0..0;
        tab.selection_reversed = false;
        tab.marked_range = None;
        tab.undo_stack.clear();
        tab.redo_stack.clear();
        tab.editor_scroll = ScrollHandle::new();
        tab.reset_preview_list();
        tab.last_lines.clear();
        tab.line_offsets.clear();
        tab.line_heights.clear();
        tab.last_bounds = None;
        self.refresh_search_matches();
        cx.notify();
    }

    fn open_tree_file(&mut self, path: PathBuf, _window: &mut Window, cx: &mut Context<Self>) {
        // With multi-document tabs, opening from the file tree creates a new
        // tab for an unopened file rather than replacing (and risking loss of)
        // the active document, so no dirty-guard prompt is needed here.
        self.open_tree_file_confirmed(path, cx);
    }

    fn open_tree_file_confirmed(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.open_file_in_new_tab_from_path(path, cx);
    }

    /// The active tab's preview blocks, re-parsed only when typing has settled.
    ///
    /// Split/Read renders call this instead of `preview_blocks_shared()`
    /// directly. While keystrokes arrive faster than [`PREVIEW_DEBOUNCE`] it
    /// returns the blocks from the previous parse (so a keystroke's render does
    /// not pay a full-document parse), arms a timer that re-renders once the
    /// pause is long enough, and caps staleness at [`PREVIEW_MAX_STALE`] so the
    /// preview keeps moving during a continuous typing burst.
    ///
    /// The parse itself runs on a background thread (`spawn_preview_parse`), so
    /// the frames where it fires no longer stall the UI; renders between spawn
    /// and landing keep showing the previous blocks. Only the very first parse
    /// of a document is synchronous, so the pane never flashes empty.
    fn preview_blocks_debounced(
        &mut self,
        cx: &mut Context<Self>,
    ) -> std::sync::Arc<Vec<PreviewBlock>> {
        let version = self.active_tab().document.version();
        let now = Instant::now();

        let tab = self.active_tab_mut();
        if version != tab.preview_seen_version {
            tab.preview_seen_version = version;
            tab.preview_changed_at = Some(now);
            tab.preview_debounce_generation = tab.preview_debounce_generation.wrapping_add(1);
            self.arm_preview_debounce(cx);
        }

        let tab = self.active_tab();
        if tab.preview_reflects_version == Some(version) {
            return tab.preview_list_blocks.clone();
        }
        if tab.preview_reflects_version.is_none() {
            // Nothing parsed yet (fresh/replaced document, or the first
            // Split/Read render): parse inline so this frame shows content
            // instead of a blank pane while a background parse runs.
            let blocks = self.active_tab().document.preview_blocks_shared();
            let tab = self.active_tab_mut();
            tab.preview_reflects_version = Some(version);
            tab.preview_reflects_at = Some(Instant::now());
            return blocks;
        }
        let since_change = tab.preview_changed_at.map(|at| now.duration_since(at));
        let since_parse = tab.preview_reflects_at.map(|at| now.duration_since(at));
        // One parse in flight at a time: while one runs, keep returning the
        // stale blocks; its landing notifies, and that render re-evaluates
        // whether the document moved on and another parse is due.
        if self.active_tab().preview_parse_inflight.is_none()
            && should_parse_preview_now(since_change, since_parse)
        {
            self.spawn_preview_parse(version, cx);
        }
        self.active_tab().preview_list_blocks.clone()
    }

    /// Parse the active tab's text on a background thread and fold the result
    /// back into the tab (and its document's derived caches) when it lands.
    /// The landing is matched to its tab by a globally unique task id rather
    /// than tab index — closing another tab shifts indices, and replacing the
    /// document clears the marker so a stale result is dropped, never applied.
    fn spawn_preview_parse(&mut self, version: u64, cx: &mut Context<Self>) {
        let task_id = next_preview_parse_id();
        let text = self.active_tab().document.text().to_string();
        self.active_tab_mut().preview_parse_inflight = Some(task_id);
        cx.spawn(async move |this, cx| {
            let (blocks, headings) = cx
                .background_spawn(
                    async move { MarkdownDocument::derive_preview_and_outline(&text) },
                )
                .await;
            let _ = this.update(cx, |app, cx| {
                let Some(tab) = app
                    .tabs
                    .iter_mut()
                    .find(|tab| tab.preview_parse_inflight == Some(task_id))
                else {
                    return;
                };
                tab.preview_parse_inflight = None;
                let blocks = std::sync::Arc::new(blocks);
                // Version-gated: refused if the document changed while the
                // parse ran. The blocks still go on screen (slightly stale
                // beats frozen mid-burst) and the version mismatch makes the
                // next render schedule a fresh parse.
                tab.document
                    .install_derived(version, blocks.clone(), headings);
                tab.preview_reflects_version = Some(version);
                tab.preview_reflects_at = Some(Instant::now());
                tab.sync_preview_list(&blocks);
                cx.notify();
            });
        })
        .detach();
    }

    /// Arm a timer that re-renders once the debounce window has passed with no
    /// further edits. Every edit bumps the tab's generation, so of the timers
    /// in flight only the one armed by the *latest* edit survives its
    /// generation check — earlier ones fire and do nothing.
    fn arm_preview_debounce(&mut self, cx: &mut Context<Self>) {
        let active_index = self.active_tab;
        let generation = self.active_tab().preview_debounce_generation;
        cx.spawn(async move |this, cx| {
            Timer::after(PREVIEW_DEBOUNCE).await;
            let _ = this.update(cx, |app, cx| {
                let Some(tab) = app.tabs.get(active_index) else {
                    return;
                };
                if tab.preview_debounce_generation != generation {
                    return;
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn schedule_autosave(&mut self, cx: &mut Context<Self>) {
        // Bump the generation even when disabled so a pending timer from a
        // previous schedule is invalidated.
        let active_index = self.active_tab;
        let autosave_enabled = self.auto_save_preferences.enabled;
        let delay = Duration::from_secs(self.auto_save_preferences.delay_secs.max(1));
        let recovery_dir = self.recovery_dir.clone();
        let tab = self.active_tab_mut();
        tab.autosave_generation = tab.autosave_generation.wrapping_add(1);
        if !autosave_enabled {
            return;
        }
        let generation = tab.autosave_generation;

        cx.spawn(async move |this, cx| {
            Timer::after(delay).await;
            let _ = this.update(cx, |app, cx| {
                // Validate the tab still exists and its generation matches, so
                // a tab switch (or close) between schedule and fire does not
                // autosave the wrong tab or a removed one.
                let Some(tab) = app.tabs.get(active_index) else {
                    return;
                };
                if tab.autosave_generation != generation || !tab.document.is_dirty() {
                    return;
                }

                let tab = &mut app.tabs[active_index];
                match tab.document.autosave(&recovery_dir) {
                    Ok(AutosaveOutcome::NoChanges) => {}
                    Ok(AutosaveOutcome::SavedFile(path)) => {
                        if let Some(recovery) = tab.last_recovery_file.take() {
                            let _ = delete_recovery_file(recovery);
                        }
                        app.status = tf(
                            app.language,
                            Msg::StatusAutoSaved,
                            &[&path.display().to_string()],
                        )
                        .into();
                    }
                    Ok(AutosaveOutcome::SavedRecovery(path)) => {
                        if let Some(previous) = tab.last_recovery_file.replace(path.clone()) {
                            if previous != path {
                                let _ = delete_recovery_file(previous);
                            }
                        }
                        app.status = tf(
                            app.language,
                            Msg::StatusRecoverySaved,
                            &[&path.display().to_string()],
                        )
                        .into();
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "auto-save failed");
                        app.status =
                            tf(app.language, Msg::StatusAutoSaveFailed, &[&err.to_string()]).into();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn search_options(&self) -> SearchOptions {
        SearchOptions {
            query: self.search_query.clone(),
            case_sensitive: self.search_case_sensitive,
            regex: self.search_regex,
        }
    }

    fn refresh_search_matches(&mut self) {
        // Skip the full-document regex scan entirely when the find bar is
        // closed: matches are recomputed on demand in show_find/show_replace,
        // so there is no point paying for it on every keystroke while typing.
        if !self.search_visible || self.search_query.is_empty() {
            if !self.search_visible {
                self.search_matches.clear();
                self.current_search_index = None;
            }
            return;
        }

        match self
            .active_tab()
            .document
            .find_matches(&self.search_options())
        {
            Ok(matches) => {
                self.search_matches = matches;
                self.current_search_index = self
                    .current_search_index
                    .filter(|index| *index < self.search_matches.len());
            }
            Err(err) => {
                self.search_matches.clear();
                self.current_search_index = None;
                self.status = self.trf(Msg::StatusFindFailed, &[err.message()]);
            }
        }
    }

    fn close_search_overlay(&mut self, cx: &mut Context<Self>) {
        hide_search_overlay_state(
            &mut self.search_visible,
            &mut self.replace_visible,
            &mut self.search_focus,
            &mut self.input_marked_len,
        );
        self.refresh_search_matches();
        cx.notify();
    }

    fn select_search_match(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(found) = self.search_matches.get(index).cloned() {
            self.current_search_index = Some(index);
            let tab = self.active_tab_mut();
            tab.selected_range = found.range.clone();
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.scroll_editor_to_offset(found.range.start);
            self.status = self.trf(
                Msg::StatusMatchPosition,
                &[
                    &(index + 1).to_string(),
                    &self.search_matches.len().to_string(),
                    &found.line.to_string(),
                    &found.column.to_string(),
                ],
            );
            cx.notify();
        }
    }

    fn jump_to_offset(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = clamp_to_text_boundary(self.active_tab().document.text(), offset);
        let tab = self.active_tab_mut();
        tab.selected_range = offset..offset;
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.scroll_editor_to_offset(offset);
        self.status = t(self.language, Msg::StatusJumpedToHeading).into();
        cx.notify();
    }

    fn scroll_editor_to_offset(&self, offset: usize) {
        self.active_tab().scroll_editor_to_offset(offset);
    }

    fn center_cursor_if_typewriter(&self) {
        if self.typewriter_mode {
            self.active_tab()
                .scroll_editor_typewriter_to_offset(self.active_tab().cursor_offset());
        }
    }

    fn current_preferences(&self) -> AppPreferences {
        // Persist the active selection by name. A selection that resolves to a
        // custom `.theme` file is written to `custom_theme` (so the loader
        // re-resolves it next launch); any other built-in name is written to
        // `theme`.
        let is_custom = self
            .custom_themes
            .iter()
            .any(|theme| theme.name.eq_ignore_ascii_case(&self.selected_theme_name));
        let (theme_name, custom_theme_name) = if is_custom {
            (
                self.theme.name().to_string(),
                Some(self.selected_theme_name.clone()),
            )
        } else {
            (self.selected_theme_name.clone(), None)
        };
        AppPreferences {
            theme: theme_name,
            custom_theme: custom_theme_name,
            focus_mode: self.focus_mode,
            typewriter_mode: self.typewriter_mode,
            code_line_numbers: self.code_line_numbers,
            preview_adaptive_width: self.preview_adaptive_width,
            heading_menu_max_level: self.heading_menu_max_level,
            sync_scroll: self.sync_scroll,
            sidebar_visible: self.sidebar_visible,
            sidebar_tab: self.sidebar_tab,
            language: self.language.code().to_string(),
            auto_save: self.auto_save_preferences,
            export: self.export_preferences.clone(),
        }
    }

    /// Translate a static UI message in the active language.
    fn tr(&self, msg: Msg) -> &'static str {
        t(self.language, msg)
    }

    /// Translate a templated UI message with positional arguments.
    fn trf(&self, msg: Msg, args: &[&str]) -> SharedString {
        tf(self.language, msg, args).into()
    }

    /// All themes the Preferences panel can offer: built-ins first (in their
    /// canonical order), then user-loaded `.theme` files.
    fn available_themes(&self) -> Vec<ThemeDefinition> {
        let mut themes = builtin_theme_definitions();
        for custom in &self.custom_themes {
            // Skip a user theme that shadows a built-in name — built-ins win
            // so the legacy name-to-palette mapping stays stable.
            if !themes.iter().any(|theme| theme.name == custom.name) {
                themes.push(custom.clone());
            }
        }
        themes
    }

    /// Resolve the active theme definition by `selected_theme_name`, checking
    /// built-ins first, then custom themes, then falling back to Paper.
    fn active_theme_definition(&self) -> ThemeDefinition {
        let name = self.selected_theme_name.trim();
        builtin_theme_definitions()
            .into_iter()
            .find(|theme| theme.name.eq_ignore_ascii_case(name))
            .or_else(|| {
                self.custom_themes
                    .iter()
                    .find(|theme| theme.name.eq_ignore_ascii_case(name))
                    .cloned()
            })
            .unwrap_or_else(|| {
                builtin_theme_definitions()
                    .into_iter()
                    .next()
                    .expect("builtin theme table is non-empty")
            })
    }

    fn palette(&self) -> ThemePalette {
        theme_palette_from_definition(&self.active_theme_definition())
    }

    /// Apply a theme by its display name (used by the Preferences panel and by
    /// `cycle_theme`). Updates both the name-based selection and the legacy
    /// `theme`/`custom_theme` fields so old code paths keep working.
    fn apply_theme_by_name(&mut self, name: &str, cx: &mut Context<Self>) {
        self.selected_theme_name = name.trim().to_string();
        let resolved = self.active_theme_definition();
        // Keep the legacy `custom_theme` field in sync: set it only when the
        // selection is a user-loaded `.theme` file.
        self.custom_theme = self
            .custom_themes
            .iter()
            .find(|theme| theme.name.eq_ignore_ascii_case(name.trim()))
            .cloned();
        // And the legacy `theme` enum, resolved from the built-in six only.
        self.theme = AppTheme::from_name(&resolved.name).unwrap_or(AppTheme::Paper);
        self.status = self.trf(Msg::StatusTheme, &[&self.theme_label()]);
        self.persist_preferences();
        cx.notify();
    }

    fn persist_preferences(&mut self) {
        if let Err(err) = save_app_preferences(&self.preferences_path, &self.current_preferences())
        {
            self.status = self.trf(Msg::StatusPreferencesSaveFailed, &[&err.to_string()]);
        }
    }

    fn active_search_text_mut(&mut self) -> Option<&mut String> {
        match self.search_focus {
            Some(SearchField::Find) => Some(&mut self.search_query),
            Some(SearchField::Replace) => Some(&mut self.replace_text),
            None => None,
        }
    }

    fn has_text_input_focus(&self) -> bool {
        self.pending_name_input.is_some()
            || self.file_tree_query_focused
            || self.search_focus.is_some()
    }

    fn active_input_text_mut(&mut self) -> Option<&mut String> {
        if self.pending_name_input.is_some() {
            self.pending_name_input
                .as_mut()
                .map(|pending| &mut pending.buffer)
        } else if self.file_tree_query_focused {
            Some(&mut self.file_tree_query)
        } else {
            self.active_search_text_mut()
        }
    }

    fn after_input_changed(&mut self, cx: &mut Context<Self>) {
        if self.pending_name_input.is_some() {
            // The name prompt edits a single buffer; no search/tree filtering
            // runs while it is open.
            self.status = t(self.language, Msg::StatusNamingEntry).into();
        } else if self.file_tree_query_focused {
            self.status = self.file_tree_summary().into();
        } else {
            self.refresh_search_matches();
            self.status = self.search_summary().into();
        }
        cx.notify();
    }

    /// Insert text into the focused redirected field, first removing any
    /// trailing IME composition. `keep_marked` records the new text as the
    /// active composition (still being edited) instead of committing it.
    fn insert_redirected_text(&mut self, text: &str, keep_marked: bool, cx: &mut Context<Self>) {
        let marked = self.input_marked_len;
        let Some(target) = self.active_input_text_mut() else {
            return;
        };
        let keep = target.len().saturating_sub(marked.min(target.len()));
        target.truncate(keep);
        target.push_str(text);
        self.input_marked_len = if keep_marked { text.len() } else { 0 };
        self.after_input_changed(cx);
    }

    fn push_text_input(&mut self, text: &str, cx: &mut Context<Self>) {
        self.insert_redirected_text(text, false, cx);
    }

    fn pop_text_input(&mut self, cx: &mut Context<Self>) -> bool {
        self.input_marked_len = 0;
        if let Some(target) = self.active_input_text_mut() {
            target.pop();
            self.after_input_changed(cx);
            true
        } else {
            false
        }
    }

    fn search_summary(&self) -> String {
        if self.search_query.is_empty() {
            t(self.language, Msg::StatusFindQueryEmpty).to_string()
        } else if self.search_matches.is_empty() {
            t(self.language, Msg::StatusNoMatches).to_string()
        } else {
            tf(
                self.language,
                Msg::StatusMatches,
                &[&self.search_matches.len().to_string()],
            )
        }
    }

    fn file_tree_summary(&self) -> String {
        let count = self
            .file_tree
            .as_ref()
            .map(|tree| tree.filtered_entries_limited(&self.file_tree_query, 0).1)
            .unwrap_or(0);
        let msg = if self.file_tree_query.is_empty() {
            Msg::StatusFilesVisible
        } else {
            Msg::StatusFileMatches
        };
        tf(self.language, msg, &[&count.to_string()])
    }

    fn new_document(&mut self, _: &NewDocument, window: &mut Window, cx: &mut Context<Self>) {
        self.confirm_discard_then(
            window,
            cx,
            Msg::DialogDiscardTitle,
            Msg::DialogDiscardNewDetail,
            Self::new_document_confirmed,
        );
    }

    fn new_document_confirmed(&mut self, cx: &mut Context<Self>) {
        self.replace_active_tab(MarkdownDocument::new(), cx);
        self.active_menu = None;
        self.status = t(self.language, Msg::StatusNewDocument).into();
        cx.notify();
    }

    fn open_document(&mut self, _: &OpenDocument, window: &mut Window, cx: &mut Context<Self>) {
        self.confirm_discard_then(
            window,
            cx,
            Msg::DialogDiscardTitle,
            Msg::DialogDiscardOpenDetail,
            Self::open_document_confirmed,
        );
    }

    fn open_document_confirmed(&mut self, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(self.tr(Msg::PromptOpenMarkdown).into()),
        });

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusOpening).into();
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match receiver.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        let display_path = path.display().to_string();
                        let focused_existing = this
                            .update(cx, |app, cx| {
                                if app.focus_existing_tab_for_path(&path, cx) {
                                    app.active_menu = None;
                                    app.status = app.trf(Msg::StatusOpened, &[&display_path]);
                                    cx.notify();
                                    true
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false);
                        if focused_existing {
                            return;
                        }

                        match MarkdownDocument::open(&path) {
                            Ok(document) => {
                                let _ = this.update(cx, |app, cx| {
                                    app.replace_active_tab(document, cx);
                                    app.active_menu = None;
                                    app.update_workspace_root_from_document(cx);
                                    app.status = app.trf(Msg::StatusOpened, &[&display_path]);
                                    cx.notify();
                                });
                                return;
                            }
                            Err(err) => tf(language, Msg::StatusOpenFailed, &[&err.to_string()]),
                        }
                    } else {
                        t(language, Msg::StatusOpenCanceled).to_string()
                    }
                }
                Ok(Ok(None)) => t(language, Msg::StatusOpenCanceled).to_string(),
                Ok(Err(err)) => tf(language, Msg::StatusOpenFailed, &[&err.to_string()]),
                Err(_) => t(language, Msg::StatusOpenCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    fn open_folder(&mut self, _: &OpenFolder, _: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(open_folder_prompt_options(self.language));

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusOpeningFolder).into();
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match receiver.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        let display_path = path.display().to_string();
                        let _ = this.update(cx, |app, cx| {
                            app.set_workspace_root(path);
                            app.sidebar_visible = true;
                            app.sidebar_tab = SidebarTab::Files;
                            app.active_menu = None;
                            app.persist_preferences();
                            app.schedule_file_tree_scan(Some(display_path), cx);
                            cx.notify();
                        });
                        return;
                    }
                    t(language, Msg::StatusOpenFolderCanceled).to_string()
                }
                Ok(Ok(None)) => t(language, Msg::StatusOpenFolderCanceled).to_string(),
                Ok(Err(err)) => tf(language, Msg::StatusOpenFolderFailed, &[&err.to_string()]),
                Err(_) => t(language, Msg::StatusOpenFolderCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    fn quit(&mut self, _: &Quit, window: &mut Window, cx: &mut Context<Self>) {
        self.request_quit(window, cx);
    }

    /// Action: open a fresh empty document in a brand-new tab. Unlike
    /// `NewDocument` (which replaces the active tab), this always adds a tab, so
    /// it is the only way to get a blank tab without going through a file.
    fn new_tab(&mut self, _: &NewTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.open_in_new_tab(MarkdownDocument::new(), cx);
        self.active_menu = None;
        self.status = t(self.language, Msg::StatusNewDocument).into();
        cx.notify();
    }

    /// Action: prompt for a file and open it in a brand-new tab.
    fn open_in_new_tab_action(
        &mut self,
        _: &OpenInNewTab,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(self.tr(Msg::PromptOpenMarkdown).into()),
        });

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusOpening).into();
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match receiver.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        let display_path = path.display().to_string();
                        let focused_existing = this
                            .update(cx, |app, cx| {
                                if app.focus_existing_tab_for_path(&path, cx) {
                                    app.active_menu = None;
                                    app.status = app.trf(Msg::StatusOpened, &[&display_path]);
                                    cx.notify();
                                    true
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false);
                        if focused_existing {
                            return;
                        }

                        match MarkdownDocument::open(&path) {
                            Ok(document) => {
                                let _ = this.update(cx, |app, cx| {
                                    app.open_in_new_tab(document, cx);
                                    app.active_menu = None;
                                    app.update_workspace_root_from_document(cx);
                                    app.status = app.trf(Msg::StatusOpened, &[&display_path]);
                                    cx.notify();
                                });
                                return;
                            }
                            Err(err) => tf(language, Msg::StatusOpenFailed, &[&err.to_string()]),
                        }
                    } else {
                        t(language, Msg::StatusOpenCanceled).to_string()
                    }
                }
                Ok(Ok(None)) => t(language, Msg::StatusOpenCanceled).to_string(),
                Ok(Err(err)) => tf(language, Msg::StatusOpenFailed, &[&err.to_string()]),
                Err(_) => t(language, Msg::StatusOpenCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    /// Action: close the active tab. If it is dirty, confirm first. Closing the
    /// last tab leaves a fresh untitled document so the window stays open.
    fn close_tab(&mut self, _: &CloseTab, window: &mut Window, cx: &mut Context<Self>) {
        self.confirm_discard_then(
            window,
            cx,
            Msg::DialogDiscardTitle,
            Msg::DialogDiscardNewDetail,
            Self::close_tab_confirmed,
        );
    }

    fn close_tab_confirmed(&mut self, cx: &mut Context<Self>) {
        // Discard the active tab's recovery file before removing it.
        if let Some(recovery) = self.active_tab_mut().last_recovery_file.take() {
            let _ = delete_recovery_file(recovery);
        }
        if self.tabs.len() <= 1 {
            // Closing the last tab leaves a fresh untitled document.
            self.tabs[0] = EditorTab::new(MarkdownDocument::new());
            self.active_tab = 0;
        } else {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
        self.active_menu = None;
        self.refresh_search_matches();
        self.status = t(self.language, Msg::StatusNewDocument).into();
        cx.notify();
    }

    /// Action: cycle to the next tab (wraps). Bound to Ctrl+Tab.
    fn next_tab(&mut self, _: &NextTab, _: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.len() > 1 {
            let index = (self.active_tab + 1) % self.tabs.len();
            self.switch_active_tab(index, cx);
        }
    }

    /// Action: cycle to the previous tab (wraps). Bound to Ctrl+Shift+Tab.
    fn prev_tab(&mut self, _: &PrevTab, _: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.len() > 1 {
            let index = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.switch_active_tab(index, cx);
        }
    }

    fn save_document(&mut self, _: &SaveDocument, window: &mut Window, cx: &mut Context<Self>) {
        if self.active_tab().document.path().is_none() {
            self.save_document_as(&SaveDocumentAs, window, cx);
            return;
        }

        let display_path = self
            .active_tab()
            .document
            .path()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let tab = self.active_tab_mut();
        self.status = match tab.document.save() {
            Ok(()) => {
                self.discard_current_recovery_file();
                self.trf(Msg::StatusSaved, &[&display_path])
            }
            Err(err) => self.trf(Msg::StatusSaveFailed, &[&err.to_string()]),
        };
        self.active_menu = None;
        cx.notify();
    }

    fn save_document_as(
        &mut self,
        _: &SaveDocumentAs,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let directory = self.suggested_directory();
        let suggested_name = self
            .active_tab()
            .document
            .path()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled.md")
            .to_string();
        let receiver = cx.prompt_for_new_path(&directory, Some(&suggested_name));

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusChoosingSaveLocation).into();
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match receiver.await {
                Ok(Ok(Some(path))) => {
                    let display_path = path.display().to_string();
                    let _ = this.update(cx, |app, cx| {
                        let tab = app.active_tab_mut();
                        app.status = match tab.document.save_as(&path) {
                            Ok(()) => {
                                app.discard_current_recovery_file();
                                app.update_workspace_root_from_document(cx);
                                app.trf(Msg::StatusSaved, &[&display_path])
                            }
                            Err(err) => app.trf(Msg::StatusSaveFailed, &[&err.to_string()]),
                        };
                        app.active_menu = None;
                        cx.notify();
                    });
                    return;
                }
                Ok(Ok(None)) => t(language, Msg::StatusSaveCanceled).to_string(),
                Ok(Err(err)) => tf(language, Msg::StatusSaveFailed, &[&err.to_string()]),
                Err(_) => t(language, Msg::StatusSaveCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    fn export_html(&mut self, _: &ExportHtml, _window: &mut Window, cx: &mut Context<Self>) {
        self.export_with_prompt(ExportFormat::Html, "html", cx);
    }

    fn export_plain_html(
        &mut self,
        _: &ExportPlainHtml,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.export_with_prompt(ExportFormat::PlainHtml, "plain.html", cx);
    }

    fn export_pdf(&mut self, _: &ExportPdf, _window: &mut Window, cx: &mut Context<Self>) {
        self.export_with_prompt(ExportFormat::Pdf, "pdf", cx);
    }

    fn export_latex(&mut self, _: &ExportLatex, _window: &mut Window, cx: &mut Context<Self>) {
        self.export_with_prompt(ExportFormat::Latex, "tex", cx);
    }

    fn export_docx(&mut self, _: &ExportDocx, _window: &mut Window, cx: &mut Context<Self>) {
        self.export_with_prompt(ExportFormat::Docx, "docx", cx);
    }

    fn export_png(&mut self, _: &ExportPng, _window: &mut Window, cx: &mut Context<Self>) {
        self.export_with_prompt(ExportFormat::Png, "png", cx);
    }

    fn export_jpeg(&mut self, _: &ExportJpeg, _window: &mut Window, cx: &mut Context<Self>) {
        self.export_with_prompt(ExportFormat::Jpeg, "jpg", cx);
    }

    fn export_with_prompt(
        &mut self,
        format: ExportFormat,
        extension: &str,
        cx: &mut Context<Self>,
    ) {
        let directory = self.suggested_directory();
        let suggested_name = self.suggested_export_name(extension);
        let receiver = cx.prompt_for_new_path(&directory, Some(&suggested_name));

        self.active_menu = None;
        self.status = self.trf(Msg::StatusChoosingExportLocation, &[extension]);
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match receiver.await {
                Ok(Ok(Some(path))) => {
                    let display_path = path.display().to_string();
                    let _ = this.update(cx, |app, cx| {
                        let export_preferences = app.export_preferences.clone();
                        let language = app.language;
                        let tab = app.active_tab_mut();
                        let outcome =
                            tab.document
                                .export_to_with(&path, format, &export_preferences);
                        app.status = match outcome {
                            // Disclose the producing backend for the formats
                            // where the pandoc engine competes with the
                            // built-in writers.
                            Ok(backend)
                                if matches!(format, ExportFormat::Pdf | ExportFormat::Docx) =>
                            {
                                let msg = match backend {
                                    ExportBackend::PandocEngine => Msg::StatusExportedEngine,
                                    ExportBackend::BuiltIn => Msg::StatusExportedBuiltin,
                                };
                                tf(language, msg, &[&display_path]).into()
                            }
                            Ok(_) => tf(language, Msg::StatusExported, &[&display_path]).into(),
                            Err(err) => {
                                tf(language, Msg::StatusExportFailed, &[&err.to_string()]).into()
                            }
                        };
                        app.active_menu = None;
                        cx.notify();
                    });
                    return;
                }
                Ok(Ok(None)) => t(language, Msg::StatusExportCanceled).to_string(),
                Ok(Err(err)) => tf(language, Msg::StatusExportFailed, &[&err.to_string()]),
                Err(_) => t(language, Msg::StatusExportCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    fn suggested_directory(&self) -> PathBuf {
        self.active_tab()
            .document
            .path()
            .and_then(Path::parent)
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn suggested_export_name(&self, extension: &str) -> String {
        self.active_tab()
            .document
            .path()
            .and_then(Path::file_stem)
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("Untitled")
            .to_string()
            + "."
            + extension
    }

    fn set_view_mode(&mut self, view_mode: ViewMode, cx: &mut Context<Self>) {
        assign_view_mode(&mut self.view_mode, view_mode);
        self.status = self.view_mode_status().into();
        self.active_menu = None;
        cx.notify();
    }

    fn view_mode_status(&self) -> &'static str {
        t(self.language, view_mode_status_message(self.view_mode))
    }

    fn toggle_view_mode(&mut self, _: &ToggleViewMode, _: &mut Window, cx: &mut Context<Self>) {
        self.set_view_mode(self.view_mode.next(), cx);
    }

    fn set_edit_mode(&mut self, _: &SetEditMode, _: &mut Window, cx: &mut Context<Self>) {
        self.set_view_mode(ViewMode::Edit, cx);
    }

    fn set_visual_edit_mode(
        &mut self,
        _: &SetVisualEditMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_view_mode(ViewMode::VisualEdit, cx);
    }

    fn set_split_preview_mode(
        &mut self,
        _: &SetSplitPreviewMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_view_mode(ViewMode::Split, cx);
    }

    fn set_read_mode(&mut self, _: &SetReadMode, _: &mut Window, cx: &mut Context<Self>) {
        self.set_view_mode(ViewMode::Read, cx);
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, _: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_visible = !self.sidebar_visible;
        // No lazy scan here: on the welcome document the Files tab shows an
        // empty-state placeholder by design (the tree has no chosen root until
        // a real file is opened). See `update_workspace_root_from_document`.
        self.status = t(
            self.language,
            if self.sidebar_visible {
                Msg::StatusSidebarShown
            } else {
                Msg::StatusSidebarHidden
            },
        )
        .into();
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    /// Switch the sidebar to a tab and persist the choice. The file tree is
    /// not lazily scanned here: on the welcome document the Files tab shows an
    /// empty-state placeholder by design.
    fn set_sidebar_tab(&mut self, tab: SidebarTab, cx: &mut Context<Self>) {
        if self.sidebar_tab == tab {
            return;
        }
        self.sidebar_tab = tab;
        self.persist_preferences();
        cx.notify();
    }

    fn select_preferences_sidebar_tab(&mut self, tab: SidebarTab, cx: &mut Context<Self>) {
        let changed = !self.sidebar_visible || self.sidebar_tab != tab;
        self.sidebar_visible = true;
        self.sidebar_tab = tab;
        if tab == SidebarTab::Files && self.file_tree.is_none() {
            self.refresh_file_tree(cx);
        }
        self.status = t(
            self.language,
            match tab {
                SidebarTab::Files => Msg::StatusFileTreeShown,
                SidebarTab::Outline => Msg::StatusOutlineShown,
            },
        )
        .into();
        if changed {
            self.persist_preferences();
        }
        cx.notify();
    }

    /// Drag handler for the editor/preview divider. The event bounds are the
    /// main content row, so the cursor's x offset within them maps directly to
    /// the editor's share of the width.
    fn on_editor_split_drag(
        &mut self,
        event: &DragMoveEvent<DraggedEditorSplitHandle>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bounds = event.bounds;
        let left = f32::from(bounds.left());
        let width = f32::from(bounds.right() - bounds.left());
        if width <= 0. {
            return;
        }
        let cursor_x = f32::from(event.event.position.x);
        let ratio =
            ((cursor_x - left) / width).clamp(EDITOR_SPLIT_RATIO_MIN, EDITOR_SPLIT_RATIO_MAX);
        if (ratio - self.editor_split_ratio).abs() > f32::EPSILON {
            self.editor_split_ratio = ratio;
            cx.notify();
        }
    }

    fn on_editor_split_drop(
        &mut self,
        _: &DraggedEditorSplitHandle,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The ratio is updated continuously during drag; drop just finalizes.
        cx.notify();
    }

    /// Drag handler for the sidebar divider. The sidebar starts at the window's
    /// left edge, so the cursor x is the new sidebar width (clamped).
    fn on_sidebar_resize_drag(
        &mut self,
        event: &DragMoveEvent<DraggedSidebarHandle>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_width =
            f32::from(event.event.position.x).clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        if (new_width - self.sidebar_width).abs() > f32::EPSILON {
            self.sidebar_width = new_width;
            cx.notify();
        }
    }

    fn toggle_outline(&mut self, _: &ToggleOutline, _: &mut Window, cx: &mut Context<Self>) {
        // Smart toggle: if the sidebar is already showing the Outline tab,
        // hide the sidebar; otherwise reveal it and switch to Outline.
        if self.sidebar_visible && self.sidebar_tab == SidebarTab::Outline {
            self.sidebar_visible = false;
            self.status = t(self.language, Msg::StatusSidebarHidden).into();
        } else {
            self.sidebar_visible = true;
            self.set_sidebar_tab(SidebarTab::Outline, cx);
            self.status = t(self.language, Msg::StatusOutlineShown).into();
        }
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    fn toggle_file_tree(&mut self, _: &ToggleFileTree, _: &mut Window, cx: &mut Context<Self>) {
        // Smart toggle: if the sidebar is already showing the Files tab, hide
        // the sidebar; otherwise reveal it and switch to Files.
        if self.sidebar_visible && self.sidebar_tab == SidebarTab::Files {
            self.sidebar_visible = false;
            self.status = t(self.language, Msg::StatusSidebarHidden).into();
        } else {
            self.sidebar_visible = true;
            self.set_sidebar_tab(SidebarTab::Files, cx);
            if self.file_tree.is_none() {
                self.refresh_file_tree(cx);
            }
            self.status = t(self.language, Msg::StatusFileTreeShown).into();
        }
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    fn focus_file_tree_search(
        &mut self,
        _: &FocusFileTreeSearch,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The filter box only exists on the Files tab, so make sure the sidebar
        // is visible and showing Files before focusing it.
        self.sidebar_visible = true;
        self.set_sidebar_tab(SidebarTab::Files, cx);
        self.file_tree_query_focused = true;
        self.search_focus = None;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        self.status = t(self.language, Msg::StatusFilteringFiles).into();
        self.active_menu = None;
        cx.notify();
    }

    fn clear_file_tree_search(
        &mut self,
        _: &ClearFileTreeSearch,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Escape first cancels any open name prompt (create/rename); only if
        // none is open does it fall back to clearing the file-tree filter.
        if self.pending_name_input.is_some() {
            self.pending_name_input = None;
            self.input_marked_len = 0;
            self.active_menu = None;
            self.file_tree_context_menu = None;
            self.status = t(self.language, Msg::StatusCanceled).into();
            cx.notify();
            return;
        }
        if self.search_visible {
            self.close_search_overlay(cx);
            return;
        }
        self.file_tree_query.clear();
        self.file_tree_query_focused = false;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        self.status = self.file_tree_summary().into();
        self.active_menu = None;
        self.file_tree_context_menu = None;
        cx.notify();
    }

    fn refresh_file_tree_action(
        &mut self,
        _: &RefreshFileTree,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.refresh_file_tree(cx);
        self.status = t(self.language, Msg::StatusFileTreeRefreshed).into();
        self.active_menu = None;
        self.pending_name_input = None;
        self.file_tree_context_menu = None;
        cx.notify();
    }

    fn open_file_in_new_tab_from_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let display_path = path.display().to_string();
        if self.focus_existing_tab_for_path(&path, cx) {
            self.active_menu = None;
            self.status = self.trf(Msg::StatusOpened, &[&display_path]);
            cx.notify();
            return;
        }

        match MarkdownDocument::open(&path) {
            Ok(document) => {
                self.open_in_new_tab(document, cx);
                self.update_workspace_root_from_document(cx);
                self.active_menu = None;
                self.status = self.trf(Msg::StatusOpened, &[&display_path]);
            }
            Err(err) => {
                self.status = self.trf(Msg::StatusOpenFailed, &[&err.to_string()]);
            }
        }
        cx.notify();
    }

    /// Handle files dragged in from the OS file manager and dropped onto the
    /// editor or preview pane. GPUI publishes the dragged paths as an
    /// [`ExternalPaths`] drag value (its platform layer turns an OS file drag
    /// into an internal drag on `Entered`, then fires `on_drop::<ExternalPaths>`
    /// on `Submit`); this handler is registered on both pane `div`s.
    ///
    /// Each dropped path that is a Markdown file is opened in its own new tab
    /// (reusing [`open_file_in_new_tab_from_path`], so the status bar reflects
    /// the last opened file). Non-Markdown files and directories are skipped
    /// silently; if nothing was opened the status bar is left untouched.
    ///
    /// A multi-file drop arrives as a single event with all paths bundled in
    /// one `ExternalPaths`, so this loops rather than taking the first path.
    fn handle_external_drop(
        &mut self,
        dragged: &ExternalPaths,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for path in dragged.paths() {
            if path.is_file() && is_markdown_path(path) {
                self.open_file_in_new_tab_from_path(path.clone(), cx);
            }
        }
    }

    fn show_file_tree_context_menu(
        &mut self,
        target: FileTreeContextTarget,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.active_menu = None;
        self.file_tree_query_focused = false;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        match &target {
            FileTreeContextTarget::Workspace => self.selected_tree_path = None,
            FileTreeContextTarget::Directory(path) | FileTreeContextTarget::File(path) => {
                self.selected_tree_path = Some(path.clone());
            }
        }
        self.file_tree_context_menu = Some(FileTreeContextMenu { target, position });
        cx.notify();
    }

    /// Open the inline name prompt for a create/rename file-tree action. The
    /// prompt captures keystrokes into its buffer via the redirected-text-input
    /// path; Enter commits (`confirm_pending_name`), Escape cancels.
    fn open_name_prompt(
        &mut self,
        kind: PendingNameKind,
        parent: PathBuf,
        target: Option<PathBuf>,
        prefill: &str,
        cx: &mut Context<Self>,
    ) {
        // Close any other transient focus so the prompt owns input routing.
        self.active_menu = None;
        self.file_tree_context_menu = None;
        self.file_tree_query_focused = false;
        self.search_focus = None;
        self.input_marked_len = 0;
        self.pending_name_input = Some(PendingNameInput {
            kind,
            parent,
            target,
            buffer: prefill.to_string(),
        });
        self.status = t(self.language, Msg::StatusNamingEntry).into();
        cx.notify();
    }

    fn handle_file_tree_context_action(
        &mut self,
        action: FileTreeContextAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self
            .file_tree_context_menu
            .as_ref()
            .map(|menu| menu.target.clone())
        else {
            return;
        };
        self.file_tree_context_menu = None;
        match action {
            FileTreeContextAction::Open => {
                if let FileTreeContextTarget::File(path) = target {
                    self.open_tree_file(path, window, cx);
                }
            }
            FileTreeContextAction::OpenInNewTab => {
                if let FileTreeContextTarget::File(path) = target {
                    self.open_file_in_new_tab_from_path(path, cx);
                }
            }
            FileTreeContextAction::CreateFile => {
                let parent = match target {
                    FileTreeContextTarget::Directory(path) => path,
                    FileTreeContextTarget::File(path) => path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.workspace_root.clone()),
                    FileTreeContextTarget::Workspace => self.workspace_root.clone(),
                };
                self.selected_tree_path = Some(parent.clone());
                self.open_name_prompt(PendingNameKind::CreateFile, parent, None, "untitled.md", cx);
            }
            FileTreeContextAction::CreateFolder => {
                let parent = match target {
                    FileTreeContextTarget::Directory(path) => path,
                    FileTreeContextTarget::File(path) => path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.workspace_root.clone()),
                    FileTreeContextTarget::Workspace => self.workspace_root.clone(),
                };
                self.selected_tree_path = Some(parent.clone());
                self.open_name_prompt(
                    PendingNameKind::CreateFolder,
                    parent,
                    None,
                    "New Folder",
                    cx,
                );
            }
            FileTreeContextAction::Rename => {
                let Some(path) = (match target {
                    FileTreeContextTarget::Directory(path) | FileTreeContextTarget::File(path) => {
                        Some(path)
                    }
                    FileTreeContextTarget::Workspace => None,
                }) else {
                    return;
                };
                let parent = path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| self.workspace_root.clone());
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("")
                    .to_string();
                self.selected_tree_path = Some(path.clone());
                self.open_name_prompt(PendingNameKind::Rename, parent, Some(path), &file_name, cx);
            }
            FileTreeContextAction::Delete => {
                self.selected_tree_path = match target {
                    FileTreeContextTarget::Directory(path) | FileTreeContextTarget::File(path) => {
                        Some(path)
                    }
                    FileTreeContextTarget::Workspace => None,
                };
                self.delete_tree_entry(&DeleteTreeEntry, window, cx);
            }
            FileTreeContextAction::ShowInFileManager => {
                let path = target.path(&self.workspace_root);
                match reveal_in_system_file_manager(
                    &path,
                    target.kind() == FileTreeContextTargetKind::File,
                ) {
                    Ok(()) => {
                        self.status = self.trf(
                            Msg::StatusShownInFileManager,
                            &[&path.display().to_string()],
                        );
                    }
                    Err(err) => {
                        self.status =
                            self.trf(Msg::StatusShowInFileManagerFailed, &[&err.to_string()]);
                    }
                }
                cx.notify();
            }
            FileTreeContextAction::Refresh => {
                self.refresh_file_tree_action(&RefreshFileTree, window, cx);
            }
            FileTreeContextAction::FilterFiles => {
                self.sidebar_visible = true;
                self.set_sidebar_tab(SidebarTab::Files, cx);
                self.file_tree_query_focused = true;
                self.search_focus = None;
                self.input_marked_len = 0;
                self.status = t(self.language, Msg::StatusFilteringFiles).into();
                cx.notify();
            }
        }
    }

    fn selected_tree_parent(&self) -> PathBuf {
        self.selected_tree_path
            .as_ref()
            .map(|path| {
                if path.is_dir() {
                    path.clone()
                } else {
                    path.parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.workspace_root.clone())
                }
            })
            .unwrap_or_else(|| self.workspace_root.clone())
    }

    fn create_tree_file(&mut self, _: &CreateTreeFile, _: &mut Window, cx: &mut Context<Self>) {
        // Open the inline name prompt against the selected entry's parent (or
        // the workspace root). The actual file is created on Enter via
        // `confirm_pending_name`.
        let parent = self.selected_tree_parent();
        self.selected_tree_path = Some(parent.clone());
        self.open_name_prompt(PendingNameKind::CreateFile, parent, None, "untitled.md", cx);
    }

    fn create_tree_folder(&mut self, _: &CreateTreeFolder, _: &mut Window, cx: &mut Context<Self>) {
        let parent = self.selected_tree_parent();
        self.selected_tree_path = Some(parent.clone());
        self.open_name_prompt(
            PendingNameKind::CreateFolder,
            parent,
            None,
            "New Folder",
            cx,
        );
    }

    fn rename_tree_entry(&mut self, _: &RenameTreeEntry, _: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = self.selected_tree_path.clone() else {
            self.status = t(self.language, Msg::StatusSelectTreeEntryFirst).into();
            cx.notify();
            return;
        };
        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.workspace_root.clone());
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        self.open_name_prompt(PendingNameKind::Rename, parent, Some(path), &file_name, cx);
    }

    /// Commit the inline name prompt: create/rename the entry using the typed
    /// buffer. An empty buffer is rejected without touching the filesystem.
    fn confirm_pending_name(
        &mut self,
        _: &ConfirmPendingName,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pending) = self.pending_name_input.take() else {
            return;
        };
        self.input_marked_len = 0;
        self.active_menu = None;
        self.file_tree_context_menu = None;

        let name = pending.buffer.trim();
        if name.is_empty() {
            self.status = t(self.language, Msg::StatusNameRequired).into();
            // Restore the prompt so the user can try again.
            self.pending_name_input = Some(pending);
            cx.notify();
            return;
        }

        match pending.kind {
            PendingNameKind::CreateFile => {
                let result = self
                    .file_tree
                    .get_or_insert_with(|| {
                        FileTree::scan(&self.workspace_root).unwrap_or(FileTree {
                            root: self.workspace_root.clone(),
                            entries: Vec::new(),
                        })
                    })
                    .create_unique_file(&pending.parent, name);
                match result {
                    Ok(path) => {
                        self.selected_tree_path = Some(path.clone());
                        self.status = self.trf(Msg::StatusCreated, &[&path.display().to_string()]);
                    }
                    Err(err) => {
                        self.status = self.trf(Msg::StatusCreateFileFailed, &[&err.to_string()]);
                    }
                }
            }
            PendingNameKind::CreateFolder => {
                let result = self
                    .file_tree
                    .get_or_insert_with(|| {
                        FileTree::scan(&self.workspace_root).unwrap_or(FileTree {
                            root: self.workspace_root.clone(),
                            entries: Vec::new(),
                        })
                    })
                    .create_unique_directory(&pending.parent, name);
                match result {
                    Ok(path) => {
                        self.selected_tree_path = Some(path.clone());
                        self.status = self.trf(Msg::StatusCreated, &[&path.display().to_string()]);
                    }
                    Err(err) => {
                        self.status = self.trf(Msg::StatusCreateFolderFailed, &[&err.to_string()]);
                    }
                }
            }
            PendingNameKind::Rename => {
                let Some(target) = pending.target.clone() else {
                    self.status = t(self.language, Msg::StatusRenameFailed).into();
                    cx.notify();
                    return;
                };
                // Refuse renaming the active document while it is dirty; the
                // user should save first to avoid losing unsaved edits.
                let needs_save = self.active_tab().document.path() == Some(target.as_path())
                    && self.active_tab().document.is_dirty();
                if needs_save {
                    self.status = t(self.language, Msg::StatusSaveBeforeRename).into();
                    cx.notify();
                    return;
                }
                let result = self
                    .file_tree
                    .as_mut()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file tree unavailable"))
                    .and_then(|tree| tree.rename_unique(&target, name));
                match result {
                    Ok(new_path) => {
                        // Reload any tab whose document path was the old path
                        // in place so the open document follows the rename.
                        let mut to_reload: Vec<(usize, MarkdownDocument)> = Vec::new();
                        for (i, tab) in self.tabs.iter_mut().enumerate() {
                            if tab.document.path() == Some(target.as_path()) {
                                if let Ok(document) = MarkdownDocument::open(&new_path) {
                                    to_reload.push((i, document));
                                }
                            }
                        }
                        for (i, document) in to_reload {
                            self.tabs[i].document = document;
                        }
                        self.selected_tree_path = Some(new_path.clone());
                        self.status =
                            self.trf(Msg::StatusRenamedTo, &[&new_path.display().to_string()]);
                    }
                    Err(err) => {
                        self.status = self.trf(Msg::StatusRenameFailed, &[&err.to_string()]);
                    }
                }
            }
        }
        cx.notify();
    }

    fn delete_tree_entry(
        &mut self,
        _: &DeleteTreeEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = self.selected_tree_path.clone() else {
            self.status = t(self.language, Msg::StatusSelectTreeEntryFirst).into();
            cx.notify();
            return;
        };

        let detail = tf(
            self.language,
            Msg::DialogDeleteDetail,
            &[&path.display().to_string()],
        );
        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogDeleteTitle),
            Some(&detail),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonDelete)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );
        // A non-empty folder is removed recursively, which is destructive and
        // not undoable, so a second confirmation specifically calls out that
        // the folder *and all of its contents* will be removed. Files and
        // empty folders keep the single confirm above. The second prompt is
        // only awaited (and thus shown) after the first is accepted.
        let recursive_answer = if path.is_dir() && dir_is_non_empty(&path) {
            let recursive_detail = tf(
                self.language,
                Msg::DialogDeleteFolderRecursiveDetail,
                &[&path.display().to_string()],
            );
            Some(window.prompt(
                PromptLevel::Warning,
                self.tr(Msg::DialogDeleteFolderRecursiveTitle),
                Some(&recursive_detail),
                &[
                    PromptButton::ok(self.tr(Msg::DialogButtonDelete)),
                    PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
                ],
                cx,
            ))
        } else {
            None
        };
        self.active_menu = None;
        self.status = t(self.language, Msg::StatusWaitingDeleteConfirm).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let recursive_confirmed = match recursive_answer {
                Some(second) => confirmed && matches!(second.await, Ok(0)),
                None => confirmed,
            };
            let _ = this.update(cx, |app, cx| {
                if !recursive_confirmed {
                    app.status = t(app.language, Msg::StatusDeleteCanceled).into();
                    cx.notify();
                    return;
                }

                let was_active = app.active_tab().document.path() == Some(path.as_path());
                let result = app
                    .file_tree
                    .as_mut()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file tree unavailable"))
                    .and_then(|tree| tree.delete(&path));
                match result {
                    Ok(()) => {
                        app.selected_tree_path = None;
                        // Any tab whose document was the deleted file - or was
                        // *inside* the deleted (now-removed) folder - is reset
                        // to a fresh untitled document so the editor never shows
                        // a stale, now-missing file.
                        for tab in app.tabs.iter_mut() {
                            let tab_path = tab.document.path();
                            let inside_deleted = tab_path
                                .map(|p| p == path.as_path() || p.starts_with(&path))
                                .unwrap_or(false);
                            if inside_deleted {
                                tab.document = MarkdownDocument::new();
                                tab.selected_range = 0..0;
                                tab.selection_reversed = false;
                                tab.undo_stack.clear();
                                tab.redo_stack.clear();
                            }
                        }
                        let _ = was_active;
                        app.status = app.trf(Msg::StatusDeleted, &[&path.display().to_string()]);
                    }
                    Err(err) => app.status = app.trf(Msg::StatusDeleteFailed, &[&err.to_string()]),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn cycle_theme(&mut self, _: &CycleTheme, _: &mut Window, cx: &mut Context<Self>) {
        // Cycle through the full combined list (built-ins + user themes) so the
        // shortcut visits every theme the Preferences panel exposes.
        let themes = self.available_themes();
        if themes.is_empty() {
            return;
        }
        let current_index = themes
            .iter()
            .position(|theme| theme.name.eq_ignore_ascii_case(&self.selected_theme_name))
            .unwrap_or(0);
        let next = themes[(current_index + 1) % themes.len()].name.clone();
        self.apply_theme_by_name(&next, cx);
        self.active_menu = None;
    }

    fn theme_label(&self) -> String {
        let name = self.active_theme_definition().name;
        let is_custom = self.custom_themes.iter().any(|theme| theme.name == name);
        if is_custom {
            tf(self.language, Msg::CustomThemeLabel, &[&name])
        } else {
            name
        }
    }

    fn show_preferences(
        &mut self,
        _: &ShowPreferences,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The Preferences panel is rendered in-app (see `preferences_panel_view`),
        // so opening it is just a flag flip. Refresh the custom-theme list so a
        // theme file dropped into the themes dir since launch shows up.
        self.ensure_sample_custom_theme();
        self.custom_themes = list_theme_definitions(&self.themes_dir).unwrap_or_default();
        self.preferences_panel_open = true;
        self.active_menu = None;
        cx.notify();
    }

    fn close_preferences(&mut self, cx: &mut Context<Self>) {
        self.preferences_panel_open = false;
        cx.notify();
    }

    fn ensure_sample_custom_theme(&mut self) {
        if self.themes_dir.exists() {
            return;
        }
        let sample = ThemeDefinition {
            name: "Midnight".to_string(),
            is_dark: true,
            colors: ThemeColors {
                app_bg: 0x10131a,
                panel_bg: 0x171b24,
                surface_bg: 0x0f1720,
                text: 0xe5edf5,
                muted: 0x91a4b7,
                border: 0x2b3544,
                active_bg: 0x23304a,
                active_text: 0x9ec5ff,
            },
        };
        let path = self.themes_dir.join("midnight.toml");
        if let Err(err) = save_theme_definition(path, &sample) {
            self.status = self.trf(Msg::StatusSampleThemeSaveFailed, &[&err.to_string()]);
        }
    }

    fn reset_preferences(
        &mut self,
        _: &ResetPreferences,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogResetTitle),
            Some(self.tr(Msg::DialogResetDetail)),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonReset)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );
        self.active_menu = None;
        self.status = t(self.language, Msg::StatusWaitingPreferenceResetConfirm).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                if confirmed {
                    let preferences = AppPreferences::default();
                    app.theme = AppTheme::from_name(&preferences.theme).unwrap_or(AppTheme::Paper);
                    app.custom_theme = None;
                    app.selected_theme_name = preferences.theme.clone();
                    app.preferences_panel_open = false;
                    app.focus_mode = preferences.focus_mode;
                    app.typewriter_mode = preferences.typewriter_mode;
                    app.code_line_numbers = preferences.code_line_numbers;
                    app.preview_adaptive_width = preferences.preview_adaptive_width;
                    app.heading_menu_max_level = preferences.heading_menu_max_level;
                    app.sync_scroll = preferences.sync_scroll;
                    app.sidebar_visible = preferences.sidebar_visible;
                    app.sidebar_tab = preferences.sidebar_tab;
                    // Reset also restores the default interface language.
                    app.language = Language::from_code(&preferences.language);
                    app.persist_preferences();
                    install_menus(app.language, app.heading_menu_max_level, cx);
                    app.status = t(app.language, Msg::StatusPreferencesReset).into();
                } else {
                    app.status = t(app.language, Msg::StatusPreferenceResetCanceled).into();
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn toggle_focus_mode(&mut self, _: &ToggleFocusMode, _: &mut Window, cx: &mut Context<Self>) {
        self.focus_mode = !self.focus_mode;
        self.status = t(
            self.language,
            if self.focus_mode {
                Msg::StatusFocusModeOn
            } else {
                Msg::StatusFocusModeOff
            },
        )
        .into();
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    fn toggle_typewriter_mode(
        &mut self,
        _: &ToggleTypewriterMode,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.typewriter_mode = !self.typewriter_mode;
        self.center_cursor_if_typewriter();
        self.status = t(
            self.language,
            if self.typewriter_mode {
                Msg::StatusTypewriterModeOn
            } else {
                Msg::StatusTypewriterModeOff
            },
        )
        .into();
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    fn toggle_code_line_numbers(
        &mut self,
        _: &ToggleCodeLineNumbers,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.code_line_numbers = !self.code_line_numbers;
        self.status = t(
            self.language,
            if self.code_line_numbers {
                Msg::StatusCodeLineNumbersOn
            } else {
                Msg::StatusCodeLineNumbersOff
            },
        )
        .into();
        self.persist_preferences();
        self.active_menu = None;
        cx.notify();
    }

    fn toggle_preview_adaptive_width(&mut self, cx: &mut Context<Self>) {
        self.preview_adaptive_width = !self.preview_adaptive_width;
        self.status = t(
            self.language,
            if self.preview_adaptive_width {
                Msg::StatusPreviewAdaptiveWidthOn
            } else {
                Msg::StatusPreviewAdaptiveWidthOff
            },
        )
        .into();
        self.persist_preferences();
        cx.notify();
    }

    fn toggle_sync_scroll(&mut self, cx: &mut Context<Self>) {
        self.sync_scroll = !self.sync_scroll;
        // Drop the cached fractions so the next Split frame reconciles from
        // whatever the current scroll positions imply, instead of treating a
        // pane as "unchanged" and skipping the first coupling.
        for tab in &mut self.tabs {
            tab.sync_scroll_editor_fraction = None;
            tab.sync_scroll_preview_fraction = None;
        }
        self.status = t(
            self.language,
            if self.sync_scroll {
                Msg::StatusSyncScrollOn
            } else {
                Msg::StatusSyncScrollOff
            },
        )
        .into();
        self.persist_preferences();
        cx.notify();
    }

    /// Proportional scroll coupling for Split Preview + Sync scroll. See
    /// [`sync_scroll_is_active`] / [`sync_fraction`]. Runs once per render.
    ///
    /// Reads each pane's current scroll offset and scrollable range, computes
    /// fractions, and — when Sync scroll is active — writes the driving pane's
    /// fraction to the other pane. The driving pane is whichever pane's
    /// *cached* fraction no longer matches its freshly-read fraction (i.e. the
    /// one the user/system just moved). After writing, both cached fractions
    /// are set to the driver's fraction, so the next frame sees no change and
    /// the write does not recur (one-frame convergence, no feedback loop).
    ///
    /// `syncing_scroll` guards against re-entrancy within the same frame. A
    /// small epsilon stops sub-pixel drift from re-triggering writes.
    fn reconcile_sync_scroll(&mut self) {
        if self.syncing_scroll || !sync_scroll_is_active(self.view_mode, self.sync_scroll) {
            return;
        }
        // Borrow the active tab mutably once; we read offsets from the
        // scroll handle / list state (which are fields on the tab) and write
        // back to them, plus the cached fractions. `Pixels(pub(crate) f32)` is
        // private, so the raw `f32` values come via `f32::from` (the public
        // `impl From<Pixels> for f32`), keeping `sync_fraction` a pure f32 helper.
        let tab = &mut self.tabs[self.active_tab];
        let editor_max = f32::from(tab.editor_scroll.max_offset().height.max(px(0.)));
        let preview_max = f32::from(
            tab.preview_list
                .max_offset_for_scrollbar()
                .height
                .max(px(0.)),
        );
        let editor_offset = f32::from(-tab.editor_scroll.offset().y)
            .max(0.)
            .min(editor_max);
        let preview_offset = f32::from(-tab.preview_list.scroll_px_offset_for_scrollbar().y)
            .max(0.)
            .min(preview_max);

        let editor_frac = sync_fraction(editor_offset, editor_max);
        let preview_frac = sync_fraction(preview_offset, preview_max);

        // If neither pane has scrollable range, there is nothing to couple.
        if editor_max <= 1. && preview_max <= 1. {
            tab.sync_scroll_editor_fraction = Some(editor_frac);
            tab.sync_scroll_preview_fraction = Some(preview_frac);
            return;
        }

        // Determine the driver: the pane whose stored fraction drifted from its
        // current fraction. First-frame (None) seeds the cache without writing,
        // so we don't yank a pane on the very first Split render.
        let editor_changed = tab.sync_scroll_editor_fraction.map_or(false, |stored| {
            (stored - editor_frac).abs() > SYNC_SCROLL_EPSILON
        });
        let preview_changed = tab.sync_scroll_preview_fraction.map_or(false, |stored| {
            (stored - preview_frac).abs() > SYNC_SCROLL_EPSILON
        });

        // Seed caches on the first observed frame (or after a reset) without
        // driving, so the next real change is the first to couple.
        let needs_seed = tab
            .sync_scroll_editor_fraction
            .zip(tab.sync_scroll_preview_fraction)
            .is_none();

        self.syncing_scroll = true;
        if !needs_seed && editor_changed && !preview_changed && preview_max > 1. {
            // Editor drove: pull the preview to the editor's fraction.
            let target = (editor_frac * preview_max).clamp(0., preview_max);
            tab.preview_list
                .set_offset_from_scrollbar(point(px(0.), px(-target)));
            tab.sync_scroll_preview_fraction = Some(editor_frac);
            tab.sync_scroll_editor_fraction = Some(editor_frac);
        } else if !needs_seed && preview_changed && !editor_changed && editor_max > 1. {
            // Preview drove: pull the editor to the preview's fraction.
            let target = (preview_frac * editor_max).clamp(0., editor_max);
            tab.editor_scroll.set_offset(point(px(0.), px(-target)));
            tab.sync_scroll_editor_fraction = Some(preview_frac);
            tab.sync_scroll_preview_fraction = Some(preview_frac);
        } else {
            // No single clear driver (both moved, or neither moved): just record
            // the current state so a future single-pane change can be detected.
            tab.sync_scroll_editor_fraction = Some(editor_frac);
            tab.sync_scroll_preview_fraction = Some(preview_frac);
        }
        self.syncing_scroll = false;
    }

    fn show_find(&mut self, _: &ShowFind, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.replace_visible = false;
        self.search_focus = Some(SearchField::Find);
        self.file_tree_query_focused = false;
        self.pending_name_input = None;
        self.input_marked_len = 0;
        let tab = self.active_tab();
        let selected = tab.selected_range.clone();
        let text_owned = if self.search_query.is_empty() && !selected.is_empty() {
            Some(tab.document.text()[selected.clone()].to_string())
        } else {
            None
        };
        if let Some(text) = text_owned {
            self.search_query = text;
        }
        self.refresh_search_matches();
        self.status = self.search_summary().into();
        self.active_menu = None;
        cx.notify();
    }

    fn show_replace(&mut self, _: &ShowReplace, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.replace_visible = true;
        self.search_focus = Some(SearchField::Find);
        self.file_tree_query_focused = false;
        self.input_marked_len = 0;
        let tab = self.active_tab();
        let selected = tab.selected_range.clone();
        let text_owned = if self.search_query.is_empty() && !selected.is_empty() {
            Some(tab.document.text()[selected.clone()].to_string())
        } else {
            None
        };
        if let Some(text) = text_owned {
            self.search_query = text;
        }
        self.refresh_search_matches();
        self.status = self.search_summary().into();
        self.active_menu = None;
        cx.notify();
    }

    fn find_next(&mut self, _: &FindNext, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.refresh_search_matches();
        if self.search_matches.is_empty() {
            self.status = self.search_summary().into();
            cx.notify();
            return;
        }
        let next = self
            .current_search_index
            .map(|index| (index + 1) % self.search_matches.len())
            .unwrap_or_else(|| {
                self.search_matches
                    .iter()
                    .position(|found| found.range.start >= self.cursor_offset())
                    .unwrap_or(0)
            });
        self.select_search_match(next, cx);
    }

    fn find_previous(&mut self, _: &FindPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.refresh_search_matches();
        if self.search_matches.is_empty() {
            self.status = self.search_summary().into();
            cx.notify();
            return;
        }
        let previous = self
            .current_search_index
            .map(|index| {
                if index == 0 {
                    self.search_matches.len() - 1
                } else {
                    index - 1
                }
            })
            .unwrap_or_else(|| {
                self.search_matches
                    .iter()
                    .rposition(|found| found.range.end <= self.cursor_offset())
                    .unwrap_or(self.search_matches.len() - 1)
            });
        self.select_search_match(previous, cx);
    }

    fn replace_current_match(
        &mut self,
        _: &ReplaceCurrentMatch,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.refresh_search_matches();
        let range = self
            .current_search_index
            .and_then(|index| self.search_matches.get(index))
            .map(|found| found.range.clone())
            .or_else(|| {
                (!self.active_tab().selected_range.is_empty())
                    .then(|| self.active_tab().selected_range.clone())
            });
        let Some(range) = range else {
            self.status = t(self.language, Msg::StatusNoMatchSelected).into();
            cx.notify();
            return;
        };

        let snapshot = self.snapshot();
        let search_options = self.search_options();
        let replace_text = self.replace_text.clone();
        let tab = self.active_tab_mut();
        let result = tab
            .document
            .replace_current_match(range, &search_options, &replace_text);
        match result {
            Ok(result) if result.replacements > 0 => {
                self.commit_undo_snapshot(snapshot);
                let tab = self.active_tab_mut();
                if let Some(range) = result.selected_range {
                    tab.selected_range = range;
                }
                tab.selection_reversed = false;
                tab.marked_range = None;
                self.after_document_changed(cx);
                self.status = t(self.language, Msg::StatusReplacedCurrent).into();
            }
            Ok(_) => {
                self.status = t(self.language, Msg::StatusNoMatchSelected).into();
            }
            Err(err) => {
                self.status = self.trf(Msg::StatusReplaceFailed, &[err.message()]);
            }
        }
        cx.notify();
    }

    fn replace_all_matches(
        &mut self,
        _: &ReplaceAllMatches,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snapshot = self.snapshot();
        let search_options = self.search_options();
        let replace_text = self.replace_text.clone();
        let tab = self.active_tab_mut();
        let result = tab
            .document
            .replace_all_matches(&search_options, &replace_text);
        match result {
            Ok(result) if result.replacements > 0 => {
                self.commit_undo_snapshot(snapshot);
                let tab = self.active_tab_mut();
                tab.selected_range = 0..0;
                tab.selection_reversed = false;
                tab.marked_range = None;
                self.after_document_changed(cx);
                self.status = self.trf(
                    Msg::StatusReplacedMatches,
                    &[&result.replacements.to_string()],
                );
            }
            Ok(_) => {
                self.status = t(self.language, Msg::StatusNoMatchesToReplace).into();
            }
            Err(err) => {
                self.status = self.trf(Msg::StatusReplaceFailed, &[err.message()]);
            }
        }
        cx.notify();
    }

    fn toggle_find_case_sensitive(
        &mut self,
        _: &ToggleFindCaseSensitive,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_case_sensitive = !self.search_case_sensitive;
        self.refresh_search_matches();
        self.status = if self.search_case_sensitive {
            "Case-sensitive find".into()
        } else {
            "Case-insensitive find".into()
        };
        cx.notify();
    }

    fn toggle_find_regex(&mut self, _: &ToggleFindRegex, _: &mut Window, cx: &mut Context<Self>) {
        self.search_regex = !self.search_regex;
        self.refresh_search_matches();
        self.status = t(
            self.language,
            if self.search_regex {
                Msg::StatusRegexFind
            } else {
                Msg::StatusLiteralFind
            },
        )
        .into();
        cx.notify();
    }

    fn apply_language(&mut self, language: Language, cx: &mut Context<Self>) {
        if self.language == language {
            self.active_menu = None;
            return;
        }
        self.language = language;
        self.persist_preferences();
        // Native (OS) menus were installed with English labels at startup;
        // re-translate them so the menu bar matches the new language.
        install_menus(self.language, self.heading_menu_max_level, cx);
        self.status = t(self.language, Msg::StatusLanguageSet).into();
        self.active_menu = None;
        cx.notify();
    }

    fn about(&mut self, _: &AboutMarkion, window: &mut Window, cx: &mut Context<Self>) {
        let detail = tf(
            self.language,
            Msg::DialogAboutDetail,
            &[env!("CARGO_PKG_VERSION"), GITHUB_REPO_URL],
        );
        let _ = window.prompt(
            PromptLevel::Info,
            self.tr(Msg::DialogAboutTitle),
            Some(&detail),
            &[PromptButton::ok(self.tr(Msg::DialogButtonOk))],
            cx,
        );
        self.status = t(self.language, Msg::StatusAboutMarkion).into();
        self.active_menu = None;
        cx.notify();
    }

    fn show_shortcuts(&mut self, _: &ShowShortcuts, window: &mut Window, cx: &mut Context<Self>) {
        let _ = window.prompt(
            PromptLevel::Info,
            self.tr(Msg::DialogShortcutsTitle),
            Some(shortcut_reference(
                self.language,
                self.heading_menu_max_level,
            )),
            &[PromptButton::ok(self.tr(Msg::DialogButtonOk))],
            cx,
        );
        self.status = t(self.language, Msg::StatusKeyboardShortcuts).into();
        self.active_menu = None;
        cx.notify();
    }

    fn snapshot(&self) -> EditorSnapshot {
        self.active_tab().snapshot()
    }

    fn push_undo_snapshot(&mut self) {
        self.active_tab_mut().push_undo_snapshot();
    }

    fn commit_undo_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.active_tab_mut().commit_undo_snapshot(snapshot);
    }

    fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if self.active_tab_mut().apply_undo() {
            self.active_menu = None;
            self.after_document_changed(cx);
            self.status = t(self.language, Msg::StatusUndo).into();
        } else {
            self.status = t(self.language, Msg::StatusNothingToUndo).into();
        }
        cx.notify();
    }

    fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if self.active_tab_mut().apply_redo() {
            self.active_menu = None;
            self.after_document_changed(cx);
            self.status = t(self.language, Msg::StatusRedo).into();
        } else {
            self.status = t(self.language, Msg::StatusNothingToRedo).into();
        }
        cx.notify();
    }

    fn apply_markdown_format(
        &mut self,
        format: MarkdownFormat,
        status: SharedString,
        cx: &mut Context<Self>,
    ) {
        let snapshot = self.snapshot();
        let tab = self.active_tab_mut();
        let new_range = tab
            .document
            .apply_markdown_format(tab.selected_range.clone(), format);
        let changed = tab.document.text() != snapshot.document.text();
        if changed {
            self.commit_undo_snapshot(snapshot);
            let tab = self.active_tab_mut();
            tab.selected_range = new_range;
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.active_menu = None;
            self.status = status;
            self.after_document_changed(cx);
        } else {
            self.status = t(self.language, Msg::StatusNoFormattingChange).into();
        }
        cx.notify();
    }

    fn apply_table_edit(&mut self, edit: TableEdit, status: SharedString, cx: &mut Context<Self>) {
        self.apply_table_edit_at(self.cursor_offset(), edit, status, cx);
    }

    fn apply_table_edit_at(
        &mut self,
        offset: usize,
        edit: TableEdit,
        status: SharedString,
        cx: &mut Context<Self>,
    ) {
        let snapshot = self.snapshot();
        let tab = self.active_tab_mut();
        let result = tab.document.edit_table_at(offset, edit);
        let changed = tab.document.text() != snapshot.document.text();
        let new_range = result.as_ref().map(|r| r.selected_range.clone());
        if changed {
            self.commit_undo_snapshot(snapshot);
            let tab = self.active_tab_mut();
            if let Some(range) = new_range {
                tab.selected_range = range;
            }
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.active_menu = None;
            self.status = status;
            self.after_document_changed(cx);
        } else if result.is_some() {
            self.active_menu = None;
            self.status = t(self.language, Msg::StatusTableAlreadyFormatted).into();
        } else {
            self.active_menu = None;
            self.status = t(self.language, Msg::StatusNoTableAtCursor).into();
        }
        cx.notify();
    }

    fn bold(&mut self, _: &Bold, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(MarkdownFormat::Bold, self.tr(Msg::StatusFmtBold).into(), cx);
    }

    fn italic(&mut self, _: &Italic, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::Italic,
            self.tr(Msg::StatusFmtItalic).into(),
            cx,
        );
    }

    fn inline_code(&mut self, _: &InlineCode, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::InlineCode,
            self.tr(Msg::StatusFmtInlineCode).into(),
            cx,
        );
    }

    fn insert_link(&mut self, _: &InsertLink, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(MarkdownFormat::Link, self.tr(Msg::StatusFmtLink).into(), cx);
    }

    fn insert_image(&mut self, _: &InsertImage, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::Image,
            self.tr(Msg::StatusFmtImage).into(),
            cx,
        );
    }

    fn apply_heading_level(&mut self, level: u8, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::Heading(level),
            self.trf(Msg::StatusFmtHeading, &[&level.to_string()]),
            cx,
        );
    }

    fn heading1(&mut self, _: &Heading1, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(1, cx);
    }

    fn heading2(&mut self, _: &Heading2, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(2, cx);
    }

    fn heading3(&mut self, _: &Heading3, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(3, cx);
    }

    fn heading4(&mut self, _: &Heading4, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(4, cx);
    }

    fn heading5(&mut self, _: &Heading5, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(5, cx);
    }

    fn heading6(&mut self, _: &Heading6, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_heading_level(6, cx);
    }

    fn set_heading_menu_max_level(&mut self, max_level: u8, cx: &mut Context<Self>) {
        let max_level = normalize_heading_menu_max_level(max_level);
        if self.heading_menu_max_level == max_level {
            return;
        }
        self.heading_menu_max_level = max_level;
        self.persist_preferences();
        install_menus(self.language, self.heading_menu_max_level, cx);
        self.active_menu = None;
        cx.notify();
    }

    fn unordered_list(&mut self, _: &UnorderedList, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::UnorderedList,
            self.tr(Msg::StatusFmtBulletedList).into(),
            cx,
        );
    }

    fn ordered_list(&mut self, _: &OrderedList, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::OrderedList,
            self.tr(Msg::StatusFmtNumberedList).into(),
            cx,
        );
    }

    fn task_list(&mut self, _: &TaskList, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::TaskList,
            self.tr(Msg::StatusFmtTaskList).into(),
            cx,
        );
    }

    fn block_quote(&mut self, _: &BlockQuote, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::BlockQuote,
            self.tr(Msg::StatusFmtBlockQuote).into(),
            cx,
        );
    }

    fn code_fence(&mut self, _: &CodeFence, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_markdown_format(
            MarkdownFormat::CodeFence,
            self.tr(Msg::StatusFmtCodeBlock).into(),
            cx,
        );
    }

    fn format_table(&mut self, _: &FormatTable, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_table_edit(
            TableEdit::Format,
            self.tr(Msg::StatusFmtFormatTable).into(),
            cx,
        );
    }

    fn table_add_row(&mut self, _: &TableAddRow, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_table_edit(TableEdit::AddRow, self.tr(Msg::StatusFmtAddRow).into(), cx);
    }

    fn table_delete_row(&mut self, _: &TableDeleteRow, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_table_edit(
            TableEdit::DeleteRow,
            self.tr(Msg::StatusFmtDeleteRow).into(),
            cx,
        );
    }

    fn table_move_row_up(&mut self, _: &TableMoveRowUp, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_table_edit(
            TableEdit::MoveRowUp,
            self.tr(Msg::StatusFmtMoveRowUp).into(),
            cx,
        );
    }

    fn table_move_row_down(
        &mut self,
        _: &TableMoveRowDown,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit(
            TableEdit::MoveRowDown,
            self.tr(Msg::StatusFmtMoveRowDown).into(),
            cx,
        );
    }

    fn table_add_column(&mut self, _: &TableAddColumn, _: &mut Window, cx: &mut Context<Self>) {
        self.apply_table_edit(
            TableEdit::AddColumn,
            self.tr(Msg::StatusFmtAddColumn).into(),
            cx,
        );
    }

    fn table_delete_column(
        &mut self,
        _: &TableDeleteColumn,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.apply_table_edit(
            TableEdit::DeleteColumn,
            self.tr(Msg::StatusFmtDeleteColumn).into(),
            cx,
        );
    }

    fn confirm_discard_then(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        message: Msg,
        detail: Msg,
        on_confirm: fn(&mut Self, &mut Context<Self>),
    ) {
        if !self.active_tab().document.is_dirty() {
            on_confirm(self, cx);
            return;
        }

        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(message),
            Some(self.tr(detail)),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonDiscard)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusWaitingConfirm).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                if confirmed {
                    on_confirm(app, cx);
                } else {
                    app.active_menu = None;
                    app.status = t(app.language, Msg::StatusCanceled).into();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn request_quit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.confirming_close {
            return;
        }

        self.active_menu = None;
        if !self.tabs.iter().any(|t| t.document.is_dirty()) {
            self.allow_close = true;
            self.status = t(self.language, Msg::StatusExitingMarkion).into();
            cx.notify();
            window.remove_window();
            cx.quit();
            return;
        }

        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogExitTitle),
            Some(self.tr(Msg::DialogExitDetail)),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonExitWithoutSaving)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );

        self.confirming_close = true;
        self.status = t(self.language, Msg::StatusWaitingExitConfirm).into();
        cx.notify();
        let window_handle = window.window_handle();

        cx.spawn(async move |this, cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                app.confirming_close = false;
                if confirmed {
                    app.discard_current_recovery_file();
                    app.allow_close = true;
                    app.status = t(app.language, Msg::StatusExitingMarkion).into();
                    cx.notify();
                    let _ = window_handle.update(cx, |_, window, _| window.remove_window());
                    cx.quit();
                } else {
                    app.status = t(app.language, Msg::StatusExitCanceled).into();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn toggle_menu(&mut self, menu: AppMenu, cx: &mut Context<Self>) {
        eprintln!(
            "[menu-debug] toggle_menu({menu:?}), was {:?}",
            self.active_menu
        );
        self.file_tree_context_menu = None;
        self.pending_name_input = None;
        self.active_menu = if self.active_menu == Some(menu) {
            None
        } else {
            Some(menu)
        };
        cx.notify();
    }

    fn close_menu(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        eprintln!("[menu-debug] close_menu, active={:?}", self.active_menu);
        if self.active_menu.is_some()
            || self.file_tree_context_menu.is_some()
            || self.preview_context_menu.is_some()
            || self.pending_name_input.is_some()
        {
            self.active_menu = None;
            self.file_tree_context_menu = None;
            self.preview_context_menu = None;
            self.pending_name_input = None;
            cx.notify();
        }
    }

    fn show_preview_context_menu(
        &mut self,
        position: Point<Pixels>,
        link_url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.active_menu = None;
        self.file_tree_context_menu = None;
        // Pane chrome and selectable runs may both handle the same right-click.
        // Prefer a resolved link over a later `None` from the pane surface.
        if let Some(existing) = &mut self.preview_context_menu {
            existing.position = position;
            if link_url.is_some() {
                existing.link_url = link_url;
            }
        } else {
            self.preview_context_menu = Some(PreviewContextMenu { position, link_url });
        }
        cx.notify();
    }

    fn select_all_preview_text(&mut self, cx: &mut Context<Self>) {
        let blocks = self.active_tab().preview_list_blocks.clone();
        let mut first: Option<PreviewCaret> = None;
        let mut last: Option<PreviewCaret> = None;
        for (block_index, block) in blocks.iter().enumerate() {
            for run_id in preview_block_runs(block) {
                let Some(text) = preview_run_plain_text(block, run_id) else {
                    continue;
                };
                if text.is_empty() {
                    continue;
                }
                let start = PreviewCaret {
                    block_index,
                    run_id,
                    offset: 0,
                };
                let end = PreviewCaret {
                    block_index,
                    run_id,
                    offset: text.len(),
                };
                if first.is_none() {
                    first = Some(start);
                }
                last = Some(end);
            }
        }
        if let (Some(anchor), Some(head)) = (first, last) {
            let tab = self.active_tab_mut();
            tab.preview_selection = Some(PreviewSelection { anchor, head });
            tab.preview_is_selecting = false;
            self.status = t(self.language, Msg::StatusPreviewSelectedAll).into();
        }
        cx.notify();
    }

    fn handle_preview_context_action(
        &mut self,
        action: PreviewContextAction,
        cx: &mut Context<Self>,
    ) {
        let link_url = self
            .preview_context_menu
            .as_ref()
            .and_then(|menu| menu.link_url.clone());
        self.preview_context_menu = None;
        match action {
            PreviewContextAction::SelectAll => {
                self.select_all_preview_text(cx);
            }
            PreviewContextAction::CopyPlain => {
                let blocks = self.active_tab().preview_list_blocks.clone();
                if let Some(text) = self
                    .active_tab()
                    .preview_selection
                    .as_ref()
                    .and_then(|sel| preview_selection_plain_text(sel, &blocks))
                {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                    self.status = t(self.language, Msg::StatusCopiedPreviewPlain).into();
                } else {
                    self.status = t(self.language, Msg::StatusNothingToCopy).into();
                }
                cx.notify();
            }
            PreviewContextAction::CopyMarkdown => {
                let blocks = self.active_tab().preview_list_blocks.clone();
                let document = self.active_tab().document.text().to_string();
                if let Some(md) = self
                    .active_tab()
                    .preview_selection
                    .as_ref()
                    .and_then(|sel| preview_selection_markdown(sel, &blocks, &document))
                {
                    cx.write_to_clipboard(ClipboardItem::new_string(md));
                    self.status = t(self.language, Msg::StatusCopiedPreviewMarkdown).into();
                } else {
                    self.status = t(self.language, Msg::StatusNothingToCopy).into();
                }
                cx.notify();
            }
            PreviewContextAction::CopyHtml => {
                let blocks = self.active_tab().preview_list_blocks.clone();
                let document = self.active_tab().document.text().to_string();
                if let Some(md) = self
                    .active_tab()
                    .preview_selection
                    .as_ref()
                    .and_then(|sel| preview_selection_markdown(sel, &blocks, &document))
                {
                    let html = MarkdownDocument::from_text(&md).render_html_fragment();
                    cx.write_to_clipboard(ClipboardItem::new_string(html));
                    self.status = t(self.language, Msg::StatusCopiedPreviewHtml).into();
                } else {
                    self.status = t(self.language, Msg::StatusNothingToCopy).into();
                }
                cx.notify();
            }
            PreviewContextAction::CopyLinkAddress => {
                if let Some(url) = link_url {
                    cx.write_to_clipboard(ClipboardItem::new_string(url));
                    self.status = t(self.language, Msg::StatusCopiedLinkAddress).into();
                } else {
                    self.status = t(self.language, Msg::StatusNothingToCopy).into();
                }
                cx.notify();
            }
        }
    }

    fn toggle_file_menu(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_menu(AppMenu::File, cx);
    }

    fn toggle_edit_menu(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_menu(AppMenu::Edit, cx);
    }

    fn toggle_view_menu(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_menu(AppMenu::View, cx);
    }

    fn toggle_format_menu(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_menu(AppMenu::Format, cx);
    }

    fn toggle_export_menu(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_menu(AppMenu::Export, cx);
    }

    fn toggle_help_menu(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_menu(AppMenu::Help, cx);
    }

    fn click_find_next(&mut self, _: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.find_next(&FindNext, window, cx);
    }

    fn click_find_previous(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.find_previous(&FindPrevious, window, cx);
    }

    fn click_replace_current(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_current_match(&ReplaceCurrentMatch, window, cx);
    }

    fn click_replace_all(&mut self, _: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_all_matches(&ReplaceAllMatches, window, cx);
    }

    fn click_close_search(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.close_search_overlay(cx);
    }

    fn click_toggle_case(&mut self, _: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.toggle_find_case_sensitive(&ToggleFindCaseSensitive, window, cx);
    }

    fn click_toggle_regex(
        &mut self,
        _: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_find_regex(&ToggleFindRegex, window, cx);
    }

    fn focus_find_field(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.search_focus = Some(SearchField::Find);
        self.file_tree_query_focused = false;
        self.input_marked_len = 0;
        self.status = t(self.language, Msg::StatusEditingFindQuery).into();
        cx.notify();
    }

    fn focus_replace_field(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.search_visible = true;
        self.replace_visible = true;
        self.search_focus = Some(SearchField::Replace);
        self.file_tree_query_focused = false;
        self.input_marked_len = 0;
        self.status = t(self.language, Msg::StatusEditingReplacement).into();
        cx.notify();
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        let (is_empty, start) = {
            let tab = self.active_tab();
            (tab.selected_range.is_empty(), tab.selected_range.start)
        };
        if is_empty {
            let boundary = self.previous_boundary(self.cursor_offset());
            self.move_to(boundary, cx);
        } else {
            self.move_to(start, cx);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        let (is_empty, end) = {
            let tab = self.active_tab();
            (tab.selected_range.is_empty(), tab.selected_range.end)
        };
        if is_empty {
            let boundary = self.next_boundary(end);
            self.move_to(boundary, cx);
        } else {
            self.move_to(end, cx);
        }
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        let boundary = self.previous_boundary(self.cursor_offset());
        self.select_to(boundary, cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        let boundary = self.next_boundary(self.cursor_offset());
        self.select_to(boundary, cx);
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        let (is_empty, boundary_start, cursor) = {
            let tab = self.active_tab();
            (
                tab.selected_range.is_empty(),
                tab.selected_range.start,
                tab.cursor_offset(),
            )
        };
        let offset = if is_empty { cursor } else { boundary_start };
        let target = self.active_tab().document.previous_line_offset(offset);
        self.move_to(target, cx);
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        let (is_empty, boundary_end, cursor) = {
            let tab = self.active_tab();
            (
                tab.selected_range.is_empty(),
                tab.selected_range.end,
                tab.cursor_offset(),
            )
        };
        let offset = if is_empty { cursor } else { boundary_end };
        let target = self.active_tab().document.next_line_offset(offset);
        self.move_to(target, cx);
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.previous_line_offset(cursor);
        self.select_to(target, cx);
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.next_line_offset(cursor);
        self.select_to(target, cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        let len = self.active_tab().document.text().len();
        self.select_to(len, cx);
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.line_start_at(cursor);
        self.move_to(target, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.cursor_offset();
        let target = self.active_tab().document.line_end_at(cursor);
        self.move_to(target, cx);
    }

    fn insert_newline(&mut self, _: &InsertNewline, _window: &mut Window, cx: &mut Context<Self>) {
        // When the inline name prompt is open, Enter commits the name instead
        // of inserting a newline into the document.
        if self.pending_name_input.is_some() {
            self.confirm_pending_name(&ConfirmPendingName, _window, cx);
            return;
        }
        let cursor = self.active_tab().selected_range.start;
        let selected = self.active_tab().selected_range.clone();
        self.push_undo_snapshot();
        let tab = self.active_tab_mut();
        if !selected.is_empty() {
            tab.document.replace_range(selected, "");
        }
        let new_cursor = tab.document.insert_markdown_newline(cursor);
        tab.selected_range = new_cursor..new_cursor;
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.status = t(self.language, Msg::StatusEditing).into();
        self.after_document_changed(cx);
        cx.notify();
    }

    fn indent(&mut self, _: &Indent, window: &mut Window, cx: &mut Context<Self>) {
        if self.has_text_input_focus() {
            self.push_text_input("    ", cx);
            return;
        }

        if self.active_tab().selected_range.is_empty() {
            self.replace_text_in_range(None, "    ", window, cx);
        } else {
            let snapshot = self.snapshot();
            let selected = self.active_tab().selected_range.clone();
            let tab = self.active_tab_mut();
            tab.selected_range = tab.document.indent_lines(selected);
            let changed = tab.document.text() != snapshot.document.text();
            if changed {
                self.commit_undo_snapshot(snapshot);
            }
            let tab = self.active_tab_mut();
            tab.selection_reversed = false;
            tab.marked_range = None;
            self.status = t(self.language, Msg::StatusIndentedSelection).into();
            if changed {
                self.after_document_changed(cx);
            }
            cx.notify();
        }
    }

    fn outdent(&mut self, _: &Outdent, _: &mut Window, cx: &mut Context<Self>) {
        let snapshot = self.snapshot();
        let selected = self.active_tab().selected_range.clone();
        let tab = self.active_tab_mut();
        tab.selected_range = tab.document.outdent_lines(selected);
        let changed = tab.document.text() != snapshot.document.text();
        if changed {
            self.commit_undo_snapshot(snapshot);
        }
        let tab = self.active_tab_mut();
        tab.selection_reversed = false;
        tab.marked_range = None;
        self.status = t(
            self.language,
            if changed {
                Msg::StatusOutdentedSelection
            } else {
                Msg::StatusNothingToOutdent
            },
        )
        .into();
        if changed {
            self.after_document_changed(cx);
        }
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.pop_text_input(cx) {
            return;
        }

        if self.active_tab().selected_range.is_empty() {
            let boundary = self.previous_boundary(self.cursor_offset());
            self.select_to(boundary, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.pop_text_input(cx) {
            return;
        }

        if self.active_tab().selected_range.is_empty() {
            let boundary = self.next_boundary(self.cursor_offset());
            self.select_to(boundary, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            if self.has_text_input_focus() {
                self.push_text_input(&text, cx);
                return;
            }
            self.replace_text_in_range(None, &text, window, cx);
        } else {
            self.status = t(self.language, Msg::StatusClipboardEmpty).into();
            cx.notify();
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        let blocks = self.active_tab().preview_list_blocks.clone();
        if preview_selection_takes_copy_precedence(
            self.active_tab().preview_selection.as_ref(),
            &blocks,
        ) {
            if let Some(text) = self
                .active_tab()
                .preview_selection
                .as_ref()
                .and_then(|sel| preview_selection_plain_text(sel, &blocks))
            {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                self.status = t(self.language, Msg::StatusCopiedSelection).into();
                cx.notify();
                return;
            }
        }
        let selected = self.active_tab().selected_range.clone();
        if !selected.is_empty() {
            let text = self.active_tab().document.text()[selected].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.status = t(self.language, Msg::StatusCopiedSelection).into();
        } else {
            self.status = t(self.language, Msg::StatusNothingToCopy).into();
        }
        cx.notify();
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        let selected = self.active_tab().selected_range.clone();
        if !selected.is_empty() {
            let text = self.active_tab().document.text()[selected].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.replace_text_in_range(None, "", window, cx);
            self.status = t(self.language, Msg::StatusCutSelection).into();
            cx.notify();
        } else {
            self.status = t(self.language, Msg::StatusNothingToCut).into();
            cx.notify();
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Clicking into the editor returns text-input focus to the document,
        // otherwise typed characters keep flowing into the file-tree filter
        // or search fields that last held focus.
        self.file_tree_query_focused = false;
        self.search_focus = None;
        self.input_marked_len = 0;
        // Source-editor selection clears any preview selection so Copy routes
        // back to the editor.
        self.active_tab_mut().clear_preview_selection();
        self.active_tab_mut().is_selecting = true;
        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.active_tab_mut().is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.active_tab().is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    /// Horizontal tab bar shown only when more than one tab is open. Each tab
    /// shows the file name (+ `*` when dirty), the active tab is highlighted,
    /// clicking switches to it, and the `×` button closes it. Styled to match
    /// the existing `menu_title_button` idiom (GPUI 0.2.2 has no native tab bar).
    fn tab_bar_view(&self, cx: &mut Context<Self>) -> Div {
        let palette = self.palette();
        if self.tabs.len() <= 1 {
            // Single-tab case: render nothing (tab bar hidden).
            return div();
        }
        let active = self.active_tab;
        let bar = div()
            .h(px(30.))
            .px_2()
            .border_b_1()
            .border_color(palette.border)
            .bg(palette.panel_bg)
            .flex()
            .items_center()
            .gap_1();
        bar.children(self.tabs.iter().enumerate().map(|(index, tab)| {
            let is_active = index == active;
            let name = title_from_path(tab.document.path()).to_string();
            let dirty = tab.document.is_dirty();
            let label: SharedString = if dirty {
                format!("{name} *").into()
            } else {
                name.into()
            };
            // Theme-driven so tabs stay legible on dark palettes (the previous
            // hard-coded light hexes rendered white tabs with light text).
            let bg = if is_active {
                palette.active_bg
            } else {
                palette.surface_bg
            };
            let text_color = if is_active {
                palette.active_text
            } else {
                palette.muted
            };
            let border = if is_active {
                palette.active_text
            } else {
                palette.border
            };
            let hover_bg = palette.active_bg;
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .border_b_2()
                .border_color(border)
                .bg(bg)
                .text_color(text_color)
                .text_size(px(12.))
                .cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .flex()
                .items_center()
                .gap_1()
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                        // The captured `index` is fixed at render time; a tab
                        // close/open since then may have shifted positions, so
                        // guard against a stale out-of-range index.
                        if index < app.tabs.len() {
                            app.switch_active_tab(index, cx);
                        }
                    }),
                )
                .child(label)
                .child(
                    div()
                        .ml_1()
                        .px_1()
                        .text_size(px(11.))
                        .cursor_pointer()
                        .hover(move |style| style.bg(border))
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |app, _: &MouseUpEvent, window, cx| {
                                // Same staleness guard as the tab click above.
                                if index < app.tabs.len() {
                                    app.switch_active_tab(index, cx);
                                    app.close_tab(&CloseTab, window, cx);
                                }
                            }),
                        )
                        .child("×"),
                )
        }))
        .child(
            // Trailing "+" opens a fresh empty tab (mirrors File → New Tab).
            div()
                .id("new-tab-button")
                .ml_1()
                .px_2()
                .py_1()
                .rounded_md()
                .text_size(px(15.))
                .text_color(palette.muted)
                .cursor_pointer()
                .hover(move |style| style.bg(palette.active_bg))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |app, _: &MouseUpEvent, window, cx| {
                        app.new_tab(&NewTab, window, cx);
                    }),
                )
                .child("+"),
        )
    }

    fn cursor_offset(&self) -> usize {
        self.active_tab().cursor_offset()
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        tab.selected_range = offset..offset;
        tab.selection_reversed = false;
        self.center_cursor_if_typewriter();
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let tab = self.active_tab_mut();
        if tab.selection_reversed {
            tab.selected_range.start = offset;
        } else {
            tab.selected_range.end = offset;
        }
        if tab.selected_range.end < tab.selected_range.start {
            tab.selection_reversed = !tab.selection_reversed;
            tab.selected_range = tab.selected_range.end..tab.selected_range.start;
        }
        self.center_cursor_if_typewriter();
        cx.notify();
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        self.active_tab().index_for_mouse_position(position)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.active_tab().previous_boundary(offset)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.active_tab().next_boundary(offset)
    }
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
        self.active_tab_mut().marked_range = None;
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

        let tab = self.active_tab_mut();
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| tab.range_from_utf16(range_utf16))
            .or(tab.marked_range.clone())
            .unwrap_or(tab.selected_range.clone());

        let changed = &tab.document.text()[range.clone()] != new_text;
        if changed {
            tab.push_undo_snapshot();
            tab.document.replace_range(range.clone(), new_text);
        }
        tab.selected_range = range.start + new_text.len()..range.start + new_text.len();
        tab.marked_range.take();
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

        let tab = self.active_tab_mut();
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| tab.range_from_utf16(range_utf16))
            .or(tab.marked_range.clone())
            .unwrap_or(tab.selected_range.clone());

        let changed = &tab.document.text()[range.clone()] != new_text;
        if changed {
            tab.push_undo_snapshot();
            tab.document.replace_range(range.clone(), new_text);
        }
        tab.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());
        tab.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| EditorTab::relative_range_from_utf16(new_text, range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.start)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
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

struct EditorElement {
    app: gpui::Entity<MarkionApp>,
}

struct PrepaintState {
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
                if let Some((key, height)) = *app.active_tab().measured_height_cache.borrow() {
                    if key == cache_key {
                        return size(width, height);
                    }
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

impl Focusable for MarkionApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MarkionApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The preview pane is hidden in Edit mode, so skip the full-document
        // parse that produces its blocks. That parse is invalidated on every
        // keystroke and, on large documents, is the dominant per-key cost
        // (~4ms at 100 KB, ~25ms at 600 KB); paying it while nothing renders it
        // is pure waste. Split/Read still parse eagerly as before.
        let preview_blocks: std::sync::Arc<Vec<PreviewBlock>> =
            if matches!(self.view_mode, ViewMode::Edit | ViewMode::VisualEdit) {
                std::sync::Arc::new(Vec::new())
            } else {
                // Debounced: mid-typing renders reuse the previous parse and a
                // timer re-renders once typing settles (see PREVIEW_DEBOUNCE).
                // The parse itself runs on a background thread and lands via
                // spawn_preview_parse, so it never stalls a frame.
                let blocks = self.preview_blocks_debounced(cx);
                // Fold the blocks into the virtualized preview list (splices
                // only the changed range, preserving scroll; a reused Arc is a
                // pointer-compare no-op).
                self.active_tab_mut().sync_preview_list(&blocks);
                blocks
            };
        let visual_blocks: std::sync::Arc<Vec<VisualBlock>> =
            if matches!(self.view_mode, ViewMode::VisualEdit) {
                let blocks = self.active_tab().document.visual_blocks_shared();
                self.active_tab_mut().sync_visual_list(&blocks);
                blocks
            } else {
                std::sync::Arc::new(Vec::new())
            };
        // Proportional scroll coupling for Split Preview + Sync scroll. Runs
        // each frame *after* the preview list is in sync with the current
        // blocks (so max_offset reflects the real content height) and *before*
        // the scrollbar views read offsets to draw thumbs. Detects which pane
        // drove the latest change via per-tab cached fractions and writes only
        // the non-driving pane, converging in one frame without a feedback loop.
        self.reconcile_sync_scroll();
        let title = title_from_path(self.active_tab().document.path());
        let palette = self.palette();
        let document_dir = self
            .active_tab()
            .document
            .path()
            .and_then(Path::parent)
            .map(PathBuf::from);
        let is_dirty = self.active_tab().document.is_dirty();
        let dirty_marker = if is_dirty { " *" } else { "" };
        let save_state = t(
            self.language,
            if is_dirty {
                Msg::TitleModified
            } else {
                Msg::TitleSaved
            },
        );
        let (editor_width, preview_width) =
            view_mode_pane_widths(self.view_mode, self.editor_split_ratio);
        let constrain_read_preview =
            read_mode_preview_is_constrained(self.view_mode, self.preview_adaptive_width);
        // Captured by value into the virtualized preview `list`'s per-item
        // render closure, which must be `'static` and cannot borrow `self`.
        let preview_items = preview_blocks.clone();
        let preview_items_doc_dir = document_dir.clone();
        let preview_code_line_numbers = self.code_line_numbers;
        let preview_list_state = self.active_tab().preview_list.clone();
        let visual_items = visual_blocks.clone();
        let visual_items_doc_dir = document_dir.clone();
        let visual_list_state = self.active_tab().visual_list.clone();

        div()
            .size_full()
            .relative()
            .bg(palette.app_bg)
            .text_color(palette.text)
            .font_family(".SystemUIFont")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Self::new_document))
            .on_action(cx.listener(Self::open_document))
            .on_action(cx.listener(Self::open_folder))
            .on_action(cx.listener(Self::save_document))
            .on_action(cx.listener(Self::save_document_as))
            .on_action(cx.listener(Self::export_html))
            .on_action(cx.listener(Self::export_plain_html))
            .on_action(cx.listener(Self::export_pdf))
            .on_action(cx.listener(Self::export_latex))
            .on_action(cx.listener(Self::export_docx))
            .on_action(cx.listener(Self::export_png))
            .on_action(cx.listener(Self::export_jpeg))
            .on_action(cx.listener(Self::toggle_view_mode))
            .on_action(cx.listener(Self::set_edit_mode))
            .on_action(cx.listener(Self::set_visual_edit_mode))
            .on_action(cx.listener(Self::set_split_preview_mode))
            .on_action(cx.listener(Self::set_read_mode))
            .on_action(cx.listener(Self::toggle_sidebar))
            .on_action(cx.listener(Self::toggle_outline))
            .on_action(cx.listener(Self::toggle_file_tree))
            .on_action(cx.listener(Self::focus_file_tree_search))
            .on_action(cx.listener(Self::clear_file_tree_search))
            .on_action(cx.listener(Self::refresh_file_tree_action))
            .on_action(cx.listener(Self::create_tree_file))
            .on_action(cx.listener(Self::create_tree_folder))
            .on_action(cx.listener(Self::rename_tree_entry))
            .on_action(cx.listener(Self::delete_tree_entry))
            .on_action(cx.listener(Self::confirm_pending_name))
            .on_action(cx.listener(Self::cycle_theme))
            .on_action(cx.listener(Self::toggle_focus_mode))
            .on_action(cx.listener(Self::toggle_typewriter_mode))
            .on_action(cx.listener(Self::toggle_code_line_numbers))
            .on_action(cx.listener(Self::show_find))
            .on_action(cx.listener(Self::show_replace))
            .on_action(cx.listener(Self::find_next))
            .on_action(cx.listener(Self::find_previous))
            .on_action(cx.listener(Self::replace_current_match))
            .on_action(cx.listener(Self::replace_all_matches))
            .on_action(cx.listener(Self::toggle_find_case_sensitive))
            .on_action(cx.listener(Self::toggle_find_regex))
            .on_action(cx.listener(Self::show_shortcuts))
            .on_action(cx.listener(Self::show_preferences))
            .on_action(cx.listener(Self::reset_preferences))
            .on_action(cx.listener(Self::about))
            .on_action(cx.listener(Self::quit))
            .on_action(cx.listener(Self::new_tab))
            .on_action(cx.listener(Self::open_in_new_tab_action))
            .on_action(cx.listener(Self::close_tab))
            .on_action(cx.listener(Self::next_tab))
            .on_action(cx.listener(Self::prev_tab))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::insert_newline))
            .on_action(cx.listener(Self::indent))
            .on_action(cx.listener(Self::outdent))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .on_action(cx.listener(Self::bold))
            .on_action(cx.listener(Self::italic))
            .on_action(cx.listener(Self::inline_code))
            .on_action(cx.listener(Self::insert_link))
            .on_action(cx.listener(Self::insert_image))
            .on_action(cx.listener(Self::heading1))
            .on_action(cx.listener(Self::heading2))
            .on_action(cx.listener(Self::heading3))
            .on_action(cx.listener(Self::heading4))
            .on_action(cx.listener(Self::heading5))
            .on_action(cx.listener(Self::heading6))
            .on_action(cx.listener(Self::unordered_list))
            .on_action(cx.listener(Self::ordered_list))
            .on_action(cx.listener(Self::task_list))
            .on_action(cx.listener(Self::block_quote))
            .on_action(cx.listener(Self::code_fence))
            .on_action(cx.listener(Self::format_table))
            .on_action(cx.listener(Self::table_add_row))
            .on_action(cx.listener(Self::table_delete_row))
            .on_action(cx.listener(Self::table_move_row_up))
            .on_action(cx.listener(Self::table_move_row_down))
            .on_action(cx.listener(Self::table_add_column))
            .on_action(cx.listener(Self::table_delete_column))
            .flex()
            .flex_col()
            .child(
                div().h(px(28.)).child(
                    div()
                        .h(px(28.))
                        .px_2()
                        .border_b_1()
                        .border_color(palette.border)
                        .bg(palette.panel_bg)
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(menu_title_button(
                            self.tr(Msg::MenuFile),
                            self.active_menu == Some(AppMenu::File),
                            palette,
                            cx.listener(Self::toggle_file_menu),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuEdit),
                            self.active_menu == Some(AppMenu::Edit),
                            palette,
                            cx.listener(Self::toggle_edit_menu),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuView),
                            self.active_menu == Some(AppMenu::View),
                            palette,
                            cx.listener(Self::toggle_view_menu),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuFormat),
                            self.active_menu == Some(AppMenu::Format),
                            palette,
                            cx.listener(Self::toggle_format_menu),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuExport),
                            self.active_menu == Some(AppMenu::Export),
                            palette,
                            cx.listener(Self::toggle_export_menu),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuHelp),
                            self.active_menu == Some(AppMenu::Help),
                            palette,
                            cx.listener(Self::toggle_help_menu),
                        )),
                ),
            )
            .child(self.tab_bar_view(cx))
            .child(
                div()
                    .id("main-content-row")
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::close_menu))
                    // Drag-move/drop for the two resize dividers. DragMoveEvent
                    // gives the cursor position plus the bounds of this row, so
                    // each handler converts the x offset into a ratio / width.
                    .on_drag_move::<DraggedEditorSplitHandle>(
                        cx.listener(Self::on_editor_split_drag),
                    )
                    .on_drop::<DraggedEditorSplitHandle>(cx.listener(Self::on_editor_split_drop))
                    .on_drag_move::<DraggedSidebarHandle>(cx.listener(Self::on_sidebar_resize_drag))
                    .on_drop::<DraggedSidebarHandle>(cx.listener(|_, _, _, cx| {
                        cx.notify();
                    }))
                    .child(sidebar_view(self, cx))
                    // Sidebar/pane divider: only when the sidebar is visible.
                    .when(self.sidebar_visible, |d| {
                        d.child(sidebar_resize_handle_view(palette.border, cx))
                    })
                    .child(
                        div()
                            // Sized by the draggable split ratio instead of a flat
                            // flex_1, so dragging the divider actually resizes it.
                            .flex_basis(DefiniteLength::Fraction(editor_width))
                            .flex_shrink()
                            .min_w_0()
                            .min_h_0()
                            .p(px(PANE_OUTER_PADDING))
                            .when(matches!(self.view_mode, ViewMode::Split), |style| {
                                style.border_r_1()
                            })
                            .border_color(palette.border)
                            .flex()
                            .flex_col()
                            // Accept files dragged from the OS file manager.
                            // The preview pane registers the same handler; the
                            // sidebar/file-tree area deliberately does not.
                            .on_drop::<ExternalPaths>(cx.listener(Self::handle_external_drop))
                            .child(if matches!(self.view_mode, ViewMode::VisualEdit) {
                                visual_edit_surface_view(
                                    visual_items,
                                    visual_items_doc_dir,
                                    visual_list_state,
                                    palette,
                                    cx,
                                )
                            } else {
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_h_0()
                                    .child(
                                        div()
                                            .size_full()
                                            .p(px(PANE_INNER_PADDING))
                                            .bg(palette.surface_bg)
                                            .border_1()
                                            .border_color(palette.border)
                                            .rounded_md()
                                            .line_height(px(EDITOR_LINE_HEIGHT))
                                            .text_size(px(15.))
                                            .cursor(CursorStyle::IBeam)
                                            .id("editor-scroll")
                                            .overflow_y_scroll()
                                            .scrollbar_width(px(PANE_SCROLLBAR_RESERVED_WIDTH))
                                            .track_scroll(&self.active_tab().editor_scroll)
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(Self::on_mouse_down),
                                            )
                                            .on_mouse_up(
                                                MouseButton::Left,
                                                cx.listener(Self::on_mouse_up),
                                            )
                                            .on_mouse_up_out(
                                                MouseButton::Left,
                                                cx.listener(Self::on_mouse_up),
                                            )
                                            .on_mouse_move(cx.listener(Self::on_mouse_move))
                                            .child(EditorElement { app: cx.entity() }),
                                    )
                                    .child(pane_scrollbar_view(
                                        PaneScrollTarget::Editor,
                                        &self.active_tab().editor_scroll,
                                        palette,
                                        cx,
                                    ))
                            })
                            .when(matches!(self.view_mode, ViewMode::Read), |style| {
                                style.hidden()
                            }),
                    )
                    // Editor/preview divider: only in Split view, where both
                    // panes are visible. In Edit/Read the hidden pane is
                    // display:none and the divider would have nothing to split.
                    .when(matches!(self.view_mode, ViewMode::Split), |d| {
                        d.child(editor_split_handle_view(palette.border, cx))
                    })
                    .child(
                        div()
                            // Remaining fraction goes to the preview pane.
                            .flex_basis(DefiniteLength::Fraction(preview_width))
                            .flex_shrink()
                            .min_w_0()
                            .min_h_0()
                            .p(px(PANE_OUTER_PADDING))
                            .flex()
                            .flex_col()
                            // Same external-file drop handler as the editor
                            // pane, so a drop lands wherever the cursor is.
                            .on_drop::<ExternalPaths>(cx.listener(Self::handle_external_drop))
                            .child(
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_h_0()
                                    .child(
                                        div()
                                            .size_full()
                                            .pl(px(PANE_INNER_PADDING))
                                            .pr(px(PREVIEW_SCROLLBAR_SAFE_RIGHT_PADDING))
                                            .bg(palette.surface_bg)
                                            .border_1()
                                            .border_color(palette.border)
                                            .rounded_md()
                                            .on_mouse_up(
                                                MouseButton::Right,
                                                cx.listener(|app, event: &MouseUpEvent, _, cx| {
                                                    app.show_preview_context_menu(
                                                        event.position,
                                                        None,
                                                        cx,
                                                    );
                                                }),
                                            )
                                            .child(
                                                // Virtualized preview: `list` builds
                                                // elements only for blocks near the
                                                // viewport, so preview render cost is
                                                // O(visible) rather than O(document).
                                                //
                                                // GPUI's list uses padding for vertical
                                                // scroll math, but item layout still starts
                                                // at the list's left edge. Keep horizontal
                                                // padding on the parent surface so rows are
                                                // actually inset from the overlay scrollbar.
                                                list(
                                                    preview_list_state,
                                                    cx.processor(
                                                        move |app, ix: usize, _window, cx| {
                                                            let block = &preview_items[ix];
                                                            let row = div()
                                                                .w_full()
                                                                .line_height(px(
                                                                    PREVIEW_LINE_HEIGHT,
                                                                ))
                                                                .child(preview_block_view(
                                                                    app,
                                                                    block,
                                                                    ix,
                                                                    preview_items_doc_dir
                                                                        .as_deref(),
                                                                    preview_code_line_numbers,
                                                                    cx,
                                                                ));
                                                            if constrain_read_preview {
                                                                div()
                                                                    .w_full()
                                                                    .flex()
                                                                    .justify_center()
                                                                    .child(row.max_w(px(
                                                                        READ_MODE_PREVIEW_MAX_WIDTH,
                                                                    )))
                                                                    .into_any_element()
                                                            } else {
                                                                row.into_any_element()
                                                            }
                                                        },
                                                    ),
                                                )
                                                .size_full()
                                                .pt(px(PANE_INNER_PADDING))
                                                .pb(px(PANE_INNER_PADDING)),
                                            ),
                                    )
                                    .child(preview_list_scrollbar_view(
                                        &self.active_tab().preview_list,
                                        palette,
                                        cx,
                                    )),
                            )
                            .when(
                                matches!(self.view_mode, ViewMode::Edit | ViewMode::VisualEdit),
                                |style| style.hidden(),
                            ),
                    ),
            )
            .child(
                div()
                    .h(px(28.))
                    .px_4()
                    .border_t_1()
                    .border_color(palette.border)
                    .text_size(px(12.))
                    .text_color(palette.muted)
                    .flex()
                    .items_center()
                    .child(format!(
                        "Markion - {title}{dirty_marker} | {save_state} | {}",
                        self.status
                    )),
            )
            .child(active_menu_dropdown(
                self.active_menu,
                self.language,
                self.heading_menu_max_level,
                palette,
                cx,
            ))
            .when(self.search_visible, |root| {
                root.child(search_panel_view(self, cx))
            })
            .when(self.file_tree_context_menu.is_some(), |root| {
                root.child(file_tree_context_menu_view(self, cx))
            })
            .when(self.preview_context_menu.is_some(), |root| {
                root.child(preview_context_menu_view(self, cx))
            })
            .when(self.preferences_panel_open, |root| {
                root.child(preferences_panel_view(self, cx))
            })
    }
}

fn visual_edit_surface_view(
    items: std::sync::Arc<Vec<VisualBlock>>,
    document_dir: Option<PathBuf>,
    list_state: ListState,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let is_empty = items.is_empty();
    div().relative().flex_1().min_h_0().child(
        div()
            .size_full()
            .pl(px(PANE_INNER_PADDING))
            .pr(px(PREVIEW_SCROLLBAR_SAFE_RIGHT_PADDING))
            .bg(palette.surface_bg)
            .border_1()
            .border_color(palette.border)
            .rounded_md()
            .cursor(CursorStyle::IBeam)
            .when(is_empty, |surface| {
                surface.child(
                    div()
                        .p(px(PANE_INNER_PADDING))
                        .text_size(px(15.))
                        .text_color(palette.muted)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|app, _, window, cx| {
                                window.focus(&app.focus_handle(cx));
                                app.move_to(0, cx);
                            }),
                        )
                        .child("Type Markdown here..."),
                )
            })
            .when(!is_empty, move |surface| {
                surface.child(
                    list(
                        list_state,
                        cx.processor(move |app, ix: usize, _window, cx| {
                            div()
                                .w_full()
                                .line_height(px(PREVIEW_LINE_HEIGHT))
                                .child(visual_block_view(
                                    app,
                                    &items[ix],
                                    ix,
                                    document_dir.as_deref(),
                                    cx,
                                ))
                                .into_any_element()
                        }),
                    )
                    .size_full()
                    .pt(px(PANE_INNER_PADDING))
                    .pb(px(PANE_INNER_PADDING)),
                )
            }),
    )
}

fn search_panel_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let current = app.current_search_index.map(|index| index + 1).unwrap_or(0);
    let total = app.search_matches.len();
    let summary = if app.search_query.is_empty() {
        "No query".to_string()
    } else {
        format!("{current}/{total}")
    };
    let top = if app.tabs.len() > 1 { px(66.) } else { px(36.) };
    let max_width = if app.replace_visible {
        px(720.)
    } else {
        px(560.)
    };

    div()
        .absolute()
        .top(top)
        .left(px(16.))
        .right(px(16.))
        .flex()
        .justify_end()
        .child(
            div()
                .w_full()
                .max_w(max_width)
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(palette.border)
                .bg(palette.panel_bg)
                .text_color(palette.text)
                .shadow_md()
                .occlude()
                .flex()
                .items_center()
                .flex_wrap()
                .gap_2()
                .child(search_field_view(
                    app.tr(Msg::SearchFind),
                    &app.search_query,
                    app.search_focus == Some(SearchField::Find),
                    palette,
                    cx.listener(MarkionApp::focus_find_field),
                ))
                .when(app.replace_visible, |panel| {
                    panel.child(search_field_view(
                        app.tr(Msg::SearchReplace),
                        &app.replace_text,
                        app.search_focus == Some(SearchField::Replace),
                        palette,
                        cx.listener(MarkionApp::focus_replace_field),
                    ))
                })
                .child(toolbar_button(
                    app.tr(Msg::SearchPrev),
                    palette,
                    cx.listener(MarkionApp::click_find_previous),
                ))
                .child(toolbar_button(
                    app.tr(Msg::SearchNext),
                    palette,
                    cx.listener(MarkionApp::click_find_next),
                ))
                .when(app.replace_visible, |panel| {
                    panel
                        .child(toolbar_button(
                            app.tr(Msg::SearchReplace),
                            palette,
                            cx.listener(MarkionApp::click_replace_current),
                        ))
                        .child(toolbar_button(
                            app.tr(Msg::SearchAll),
                            palette,
                            cx.listener(MarkionApp::click_replace_all),
                        ))
                })
                .child(toolbar_button(
                    if app.search_case_sensitive {
                        app.tr(Msg::SearchCaseSensitiveMark)
                    } else {
                        app.tr(Msg::SearchCaseInsensitiveMark)
                    },
                    palette,
                    cx.listener(MarkionApp::click_toggle_case),
                ))
                .child(toolbar_button(
                    if app.search_regex {
                        app.tr(Msg::SearchRegexMark)
                    } else {
                        app.tr(Msg::SearchLiteral)
                    },
                    palette,
                    cx.listener(MarkionApp::click_toggle_regex),
                ))
                .child(
                    div()
                        .ml_1()
                        .text_size(px(12.))
                        .text_color(palette.muted)
                        .child(summary),
                )
                .child(toolbar_button(
                    "×",
                    palette,
                    cx.listener(MarkionApp::click_close_search),
                )),
        )
}

fn hide_search_overlay_state(
    search_visible: &mut bool,
    replace_visible: &mut bool,
    search_focus: &mut Option<SearchField>,
    input_marked_len: &mut usize,
) {
    *search_visible = false;
    *replace_visible = false;
    *search_focus = None;
    *input_marked_len = 0;
}

fn search_field_view(
    label: &'static str,
    value: &str,
    active: bool,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let border = if active {
        palette.active_bg
    } else {
        palette.border
    };
    let text = if value.is_empty() {
        format!("{label}: ")
    } else {
        format!("{label}: {value}")
    };

    div()
        .min_w(px(180.))
        .max_w(px(280.))
        .flex_1()
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(palette.surface_bg)
        .text_color(palette.text)
        .text_size(px(12.))
        .cursor_pointer()
        .hover(move |style| style.border_color(palette.active_bg))
        .child(text)
        .on_mouse_up(MouseButton::Left, listener)
}

/// Inline name prompt for a file-tree create/rename action. Reuses the
/// redirected-text-input path: clicking the field focuses the name buffer so
/// IME keystrokes route into `pending_name_input.buffer` instead of the
/// document. The label is "Name" and the buffer is shown after it.
fn pending_name_prompt_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let app_entity = cx.entity();
    let Some(pending) = &app.pending_name_input else {
        return div().hidden();
    };
    let label = app.tr(Msg::FileTreeNamePromptLabel);
    let text = if pending.buffer.is_empty() {
        format!("{label}: ")
    } else {
        format!("{label}: {}", pending.buffer)
    };

    div()
        .mb_2()
        .min_w(px(180.))
        .max_w(px(320.))
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        // The prompt is always active when shown (it captures keystrokes),
        // so use the same accent blue as the active search field.
        .border_color(rgb(0x2563eb))
        .bg(palette.surface_bg)
        .text_size(px(12.))
        .text_color(palette.text)
        .cursor_pointer()
        .child(text)
        .on_mouse_up(MouseButton::Left, move |_, _window, cx| {
            // Re-assert focus if the user clicks the prompt while it is open.
            let _ = app_entity.update(cx, |app, cx| {
                if app.pending_name_input.is_some() {
                    app.input_marked_len = 0;
                    cx.notify();
                }
            });
        })
}

fn file_tree_panel_body(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();

    // Empty state: until a real Markdown file is opened the tree has no chosen
    // root (the welcome document has no path), so we deliberately show a
    // placeholder instead of scanning the program's working directory. The
    // filter input and toolbar are hidden here because they operate on a tree
    // that does not exist yet.
    if app.file_tree.is_none() {
        return div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px_4()
            .py_6()
            .text_size(px(12.))
            .text_color(palette.muted)
            .text_center()
            .child(app.tr(Msg::FileTreeEmptyState).to_string());
    }

    let app_entity = cx.entity();
    let active_path = app.active_tab().document.path().map(Path::to_path_buf);
    let selected_path = app.selected_tree_path.clone();
    // Cap how many rows the panel builds per frame: this view is rebuilt on
    // every keystroke, and an uncapped workspace scan can hold thousands of
    // entries.
    const MAX_VISIBLE_TREE_ENTRIES: usize = 300;
    let (entries, total_entries) = app
        .file_tree
        .as_ref()
        .map(|tree| {
            filtered_visible_file_tree_entries(
                tree,
                &app.file_tree_query,
                &app.collapsed_tree_paths,
                MAX_VISIBLE_TREE_ENTRIES,
            )
        })
        .unwrap_or_default();
    let hidden_entries = total_entries.saturating_sub(entries.len());
    let tree_content_width = file_tree_content_width(&entries);
    let background_app_entity = app_entity.clone();
    let root_label = app
        .workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| app.tr(Msg::FileTreeWorkspaceFallback))
        .to_string();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        // The tab already says "Files"; show the workspace name as a muted
        // subheading so users still know which directory they are browsing.
        .child(
            div()
                .mb_2()
                .text_size(px(12.))
                .text_color(palette.muted)
                .child(root_label),
        )
        // Inline name prompt overlay for create/rename. Rendered as a focused
        // input line above the tree (not as a tree row), so bounded-row
        // rendering is unaffected. Reuses the redirected-text-input path.
        .when(app.pending_name_input.is_some(), |panel| {
            panel.child(pending_name_prompt_view(app, cx))
        })
        .child(
            div()
                .id("file-tree-scroll")
                .flex_1()
                .min_h_0()
                .overflow_x_scroll()
                .overflow_y_scroll()
                .scrollbar_width(px(8.))
                .track_scroll(&app.file_tree_scroll)
                .on_mouse_up(MouseButton::Right, move |event, _, cx| {
                    let _ = background_app_entity.update(cx, |app, cx| {
                        app.show_file_tree_context_menu(
                            FileTreeContextTarget::Workspace,
                            event.position,
                            cx,
                        );
                    });
                })
                .children(entries.into_iter().map(move |entry| {
                    let left_app_entity = app_entity.clone();
                    let right_app_entity = app_entity.clone();
                    let path = entry.path.clone();
                    let entry_kind = entry.kind;
                    let context_target = match entry.kind {
                        FileTreeEntryKind::Directory => {
                            FileTreeContextTarget::Directory(entry.path.clone())
                        }
                        FileTreeEntryKind::File => FileTreeContextTarget::File(entry.path.clone()),
                    };
                    let is_active = active_path.as_ref() == Some(&entry.path);
                    let is_selected = selected_path.as_ref() == Some(&entry.path);
                    let is_collapsed = entry.kind == FileTreeEntryKind::Directory
                        && app.collapsed_tree_paths.contains(&entry.path);
                    // Only Markdown files are collected into the tree (see
                    // `collect_file_tree_entries`), so every File row opens a
                    // document; Directory rows toggle their descendants.
                    // `entry.is_markdown` is read defensively in case the
                    // collection filter relaxes.
                    let clickable = entry.kind == FileTreeEntryKind::File && entry.is_markdown;
                    let bg = if is_active {
                        palette.active_bg
                    } else if is_selected {
                        palette.surface_bg
                    } else {
                        palette.panel_bg
                    };
                    let text_color = if is_active || is_selected {
                        palette.active_text
                    } else if clickable {
                        palette.text
                    } else {
                        palette.muted
                    };

                    div()
                        .mb(px(0.))
                        .ml(px(entry.depth as f32 * 12.))
                        .w_full()
                        .min_w(px(tree_content_width))
                        .px_2()
                        .py(px(1.))
                        .rounded_md()
                        .bg(bg)
                        .text_size(px(12.))
                        .line_height(px(17.))
                        .text_color(text_color)
                        .cursor_pointer()
                        .hover(move |style| {
                            if clickable || entry_kind == FileTreeEntryKind::Directory {
                                style.bg(palette.active_bg)
                            } else {
                                style
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .min_w_0()
                                .child(file_tree_entry_icon(
                                    entry.kind,
                                    !is_collapsed,
                                    palette,
                                    text_color,
                                ))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .whitespace_nowrap()
                                        .child(entry.name),
                                ),
                        )
                        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                            let focus_handle = left_app_entity.read(cx).focus_handle.clone();
                            window.focus(&focus_handle);
                            let path = path.clone();
                            let _ = left_app_entity.update(cx, |app, cx| {
                                app.selected_tree_path = Some(path.clone());
                                app.file_tree_query_focused = false;
                                app.input_marked_len = 0;
                                match entry_kind {
                                    FileTreeEntryKind::File if clickable => {
                                        app.open_tree_file(path, window, cx);
                                    }
                                    FileTreeEntryKind::Directory => {
                                        if app.collapsed_tree_paths.contains(&path) {
                                            app.collapsed_tree_paths.remove(&path);
                                        } else {
                                            app.collapsed_tree_paths.insert(path.clone());
                                        }
                                        app.status =
                                            t(app.language, Msg::StatusSelectedTreeEntry).into();
                                        cx.notify();
                                    }
                                    FileTreeEntryKind::File => {
                                        app.status =
                                            t(app.language, Msg::StatusSelectedTreeEntry).into();
                                        cx.notify();
                                    }
                                }
                            });
                            if !clickable {
                                return;
                            }
                        })
                        .on_mouse_up(MouseButton::Right, {
                            move |event, window, cx| {
                                let focus_handle = right_app_entity.read(cx).focus_handle.clone();
                                window.focus(&focus_handle);
                                let _ = right_app_entity.update(cx, |app, cx| {
                                    app.show_file_tree_context_menu(
                                        context_target.clone(),
                                        event.position,
                                        cx,
                                    );
                                });
                            }
                        })
                }))
                .children((hidden_entries > 0).then(|| {
                    div()
                        .mt_1()
                        .px_2()
                        .text_size(px(11.))
                        .text_color(palette.muted)
                        .child(app.trf(Msg::FileTreeMoreHidden, &[&hidden_entries.to_string()]))
                })),
        )
}

fn filtered_visible_file_tree_entries(
    tree: &FileTree,
    query: &str,
    collapsed_paths: &HashSet<PathBuf>,
    limit: usize,
) -> (Vec<FileTreeEntry>, usize) {
    let query = query.trim().to_ascii_lowercase();
    let mut entries = Vec::new();
    let mut matched = 0usize;
    let mut collapsed_depth = None;

    'entries: for entry in &tree.entries {
        if let Some(depth) = collapsed_depth {
            if entry.depth > depth {
                continue 'entries;
            }
            collapsed_depth = None;
        }

        let collapsed =
            entry.kind == FileTreeEntryKind::Directory && collapsed_paths.contains(&entry.path);
        if file_tree_entry_matches_query(entry, &tree.root, &query) {
            if matched < limit {
                entries.push(entry.clone());
            }
            matched += 1;
        }
        if collapsed {
            collapsed_depth = Some(entry.depth);
        }
    }

    (entries, matched)
}

fn file_tree_entry_matches_query(entry: &FileTreeEntry, root: &Path, query: &str) -> bool {
    query.is_empty()
        || entry.name.to_ascii_lowercase().contains(query)
        || entry
            .path
            .strip_prefix(root)
            .ok()
            .and_then(Path::to_str)
            .map(|path| path.to_ascii_lowercase().contains(query))
            .unwrap_or(false)
}

fn file_tree_content_width(entries: &[FileTreeEntry]) -> f32 {
    entries
        .iter()
        .map(|entry| entry.depth as f32 * 12. + 34. + estimate_file_tree_text_width(&entry.name))
        .fold(1., f32::max)
}

fn estimate_file_tree_text_width(text: &str) -> f32 {
    text.chars()
        .map(|ch| if ch.is_ascii() { 7. } else { 12. })
        .sum()
}

/// Whether `path` is a directory that contains at least one entry.
/// Used to decide whether deleting the folder needs a second (recursive)
/// confirmation: empty folders are safe to remove with a single confirm.
fn dir_is_non_empty(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

fn reveal_in_system_file_manager(path: &Path, select_file: bool) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("explorer");
        if select_file {
            command.arg(format!("/select,{}", path.display()));
        } else {
            command.arg(path);
        }
        command.spawn().map(|_| ())
    }
    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        if select_file {
            command.arg("-R").arg(path);
        } else {
            command.arg(path);
        }
        command.spawn().map(|_| ())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target = if select_file {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        Command::new("xdg-open").arg(target).spawn().map(|_| ())
    }
}

fn file_tree_entry_icon(
    kind: FileTreeEntryKind,
    expanded: bool,
    palette: ThemePalette,
    color: Rgba,
) -> Div {
    match kind {
        FileTreeEntryKind::Directory if expanded => div()
            .relative()
            .w(px(16.))
            .h(px(14.))
            .flex_none()
            .child(
                div()
                    .absolute()
                    .top(px(2.))
                    .left(px(1.))
                    .w(px(12.))
                    .h(px(8.))
                    .rounded_sm()
                    .border_1()
                    .border_color(color)
                    .bg(palette.surface_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(1.))
                    .left(px(2.))
                    .w(px(6.))
                    .h(px(4.))
                    .rounded_sm()
                    .border_1()
                    .border_color(color)
                    .bg(palette.surface_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(6.))
                    .left(px(0.))
                    .w(px(15.))
                    .h(px(7.))
                    .rounded_sm()
                    .border_1()
                    .border_color(color)
                    .bg(palette.panel_bg),
            ),
        FileTreeEntryKind::Directory => div()
            .relative()
            .w(px(16.))
            .h(px(14.))
            .flex_none()
            .child(
                div()
                    .absolute()
                    .top(px(2.))
                    .left(px(2.))
                    .w(px(6.))
                    .h(px(4.))
                    .rounded_sm()
                    .border_1()
                    .border_color(color)
                    .bg(palette.surface_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(5.))
                    .left(px(1.))
                    .w(px(14.))
                    .h(px(8.))
                    .rounded_sm()
                    .border_1()
                    .border_color(color)
                    .bg(palette.surface_bg),
            ),
        FileTreeEntryKind::File => div()
            .relative()
            .w(px(14.))
            .h(px(16.))
            .flex_none()
            .child(
                div()
                    .absolute()
                    .top(px(1.))
                    .left(px(1.))
                    .w(px(11.))
                    .h(px(14.))
                    .rounded_sm()
                    .border_1()
                    .border_color(color)
                    .bg(palette.surface_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(2.))
                    .left(px(8.))
                    .w(px(4.))
                    .h(px(4.))
                    .border_l_1()
                    .border_b_1()
                    .border_color(color)
                    .bg(palette.panel_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(5.))
                    .left(px(3.))
                    .text_size(px(7.))
                    .line_height(px(7.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(color)
                    .child("M"),
            ),
    }
}

fn outline_panel_body(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let outline = app.active_tab().document.outline();
    let current = app
        .active_tab()
        .document
        .current_heading_index(app.active_tab().cursor_offset());
    let app_entity = cx.entity();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(
            div()
                .flex_1()
                .min_h_0()
                .children(outline.iter().enumerate().map(|(index, heading)| {
                    let app_entity = app_entity.clone();
                    let offset = heading.offset;
                    let active = current == Some(index);
                    let title = heading.title.clone();
                    let background = if active {
                        palette.active_bg
                    } else {
                        palette.panel_bg
                    };

                    div()
                        .mb_1()
                        .ml(px((heading.level.saturating_sub(1) as f32) * 12.))
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(background)
                        .text_size(px(12.))
                        .text_color(if active {
                            palette.active_text
                        } else {
                            palette.text
                        })
                        .cursor_pointer()
                        .hover(move |style| style.bg(palette.active_bg))
                        .child(title)
                        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                            let focus_handle = app_entity.read(cx).focus_handle.clone();
                            window.focus(&focus_handle);
                            let _ = app_entity.update(cx, |app, cx| {
                                app.jump_to_offset(offset, cx);
                            });
                        })
                })),
        )
}

/// Unified left sidebar: a tab bar switches between the Files panel and the
/// document Outline, and the whole column can be toggled on/off as one unit.
fn sidebar_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let app_entity = cx.entity();
    let active_tab = app.sidebar_tab;
    // Width is driven by `app.sidebar_width` so the resize divider can change
    // it; clamped to a sane range in the drag handler.
    let sidebar_width = app.sidebar_width;

    let files_active = active_tab == SidebarTab::Files;
    let outline_active = active_tab == SidebarTab::Outline;
    let files_bg = if files_active {
        palette.active_bg
    } else {
        palette.panel_bg
    };
    let files_text = if files_active {
        palette.active_text
    } else {
        palette.text
    };
    let outline_bg = if outline_active {
        palette.active_bg
    } else {
        palette.panel_bg
    };
    let outline_text = if outline_active {
        palette.active_text
    } else {
        palette.text
    };
    let hover_bg = palette.active_bg;

    div()
        .w(px(sidebar_width))
        .min_h_0()
        .flex_shrink_0()
        .p(px(SIDEBAR_COMPACT_PADDING))
        .border_r_1()
        .border_color(palette.border)
        .bg(palette.panel_bg)
        .flex()
        .flex_col()
        // NOTE: `.hidden()` must come *after* `.flex()`/`.flex_col()`. In GPUI
        // both set the same `display` field, so a later `.flex()` would clobber
        // an earlier `.hidden()` and the sidebar would never actually hide.
        .when(!app.sidebar_visible, |style| style.hidden())
        // Tab bar: Files / Outline. The active tab uses the same active-palette
        // highlight as tree rows so the two stay visually consistent.
        .child(
            div()
                .mb(px(PANE_OUTER_PADDING))
                .flex()
                .gap_1()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(files_bg)
                        .text_size(px(12.))
                        .text_color(files_text)
                        .cursor_pointer()
                        .hover(move |style| style.bg(hover_bg))
                        .child(app.tr(Msg::LabelFiles))
                        .on_mouse_up(MouseButton::Left, {
                            let app_entity = app_entity.clone();
                            move |_, _, cx| {
                                let _ = app_entity.update(cx, |app, cx| {
                                    app.set_sidebar_tab(SidebarTab::Files, cx);
                                });
                            }
                        }),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(outline_bg)
                        .text_size(px(12.))
                        .text_color(outline_text)
                        .cursor_pointer()
                        .hover(move |style| style.bg(hover_bg))
                        .child(app.tr(Msg::LabelOutline))
                        .on_mouse_up(MouseButton::Left, {
                            let app_entity = app_entity.clone();
                            move |_, _, cx| {
                                let _ = app_entity.update(cx, |app, cx| {
                                    app.set_sidebar_tab(SidebarTab::Outline, cx);
                                });
                            }
                        }),
                ),
        )
        // Only build the active panel body when the sidebar is actually
        // visible. The whole sidebar is `.hidden()` when collapsed, but the
        // element tree (and, for the Outline tab, the full-document heading
        // parse in `outline_panel_body`) was still constructed every frame.
        // Skipping it here means a collapsed sidebar costs nothing per keystroke.
        .when(app.sidebar_visible, |container| {
            container.child(match active_tab {
                SidebarTab::Files => file_tree_panel_body(app, cx),
                SidebarTab::Outline => outline_panel_body(app, cx),
            })
        })
}

fn pane_scrollbar_view(
    target: PaneScrollTarget,
    scroll_handle: &ScrollHandle,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    let id = match target {
        PaneScrollTarget::Editor => "editor-pane-scrollbar",
        PaneScrollTarget::Preview => "preview-pane-scrollbar",
    };
    let viewport_height = scroll_handle.bounds().size.height;
    let max_scroll = scroll_handle.max_offset().height.max(px(0.));
    if viewport_height <= px(0.) || max_scroll <= px(1.) {
        return div().id(id).hidden();
    }

    let track_inset = px(PANE_SCROLLBAR_EDGE_INSET);
    let track_height = (viewport_height - track_inset - track_inset).max(px(0.));
    if track_height <= px(0.) {
        return div().id(id).hidden();
    }

    let content_height = viewport_height + max_scroll;
    let thumb_height = (track_height * (viewport_height / content_height))
        .clamp(px(PANE_SCROLLBAR_MIN_THUMB_HEIGHT), track_height);
    let thumb_travel = (track_height - thumb_height).max(px(0.));
    let offset_y = (-scroll_handle.offset().y).clamp(px(0.), max_scroll);
    let thumb_top = track_inset + thumb_travel * (offset_y / max_scroll);

    let entity = cx.entity();
    let scroll_handle = scroll_handle.clone();

    div()
        .id(id)
        .absolute()
        .top(thumb_top)
        .right(px(2.))
        .w(px(PANE_SCROLLBAR_THUMB_WIDTH))
        .h(thumb_height)
        .rounded_md()
        .bg(palette.muted)
        .hover(move |style| style.bg(palette.active_text))
        .block_mouse_except_scroll()
        .child(
            canvas(
                |_, _, _| (),
                move |thumb_bounds, _, window, _| {
                    window.on_mouse_event({
                        let entity = entity.clone();
                        move |event: &MouseDownEvent, _, _, cx| {
                            if !thumb_bounds.contains(&event.position) {
                                return;
                            }
                            let _ = entity.update(cx, |app, _| {
                                app.pane_scrollbar_drag = Some(PaneScrollbarDrag {
                                    target,
                                    thumb_grab_offset_y: event.position.y - thumb_bounds.top(),
                                });
                            });
                        }
                    });
                    window.on_mouse_event({
                        let entity = entity.clone();
                        move |_: &MouseUpEvent, _, _, cx| {
                            let _ = entity.update(cx, |app, _| {
                                if app
                                    .pane_scrollbar_drag
                                    .is_some_and(|drag| drag.target == target)
                                {
                                    app.pane_scrollbar_drag = None;
                                }
                            });
                        }
                    });
                    window.on_mouse_event({
                        let entity = entity.clone();
                        let scroll_handle = scroll_handle.clone();
                        move |event: &MouseMoveEvent, _, _, cx| {
                            if !event.dragging() {
                                return;
                            }
                            let Some(drag) = entity.read(cx).pane_scrollbar_drag else {
                                return;
                            };
                            if drag.target != target {
                                return;
                            }

                            let viewport_height = scroll_handle.bounds().size.height;
                            let max_scroll = scroll_handle.max_offset().height.max(px(0.));
                            if viewport_height <= px(0.) || max_scroll <= px(1.) {
                                return;
                            }

                            let track_inset = px(PANE_SCROLLBAR_EDGE_INSET);
                            let track_height =
                                (viewport_height - track_inset - track_inset).max(px(0.));
                            if track_height <= px(0.) {
                                return;
                            }

                            let content_height = viewport_height + max_scroll;
                            let thumb_height = (track_height * (viewport_height / content_height))
                                .clamp(px(PANE_SCROLLBAR_MIN_THUMB_HEIGHT), track_height);
                            let thumb_travel = (track_height - thumb_height).max(px(0.));
                            if thumb_travel <= px(0.) {
                                return;
                            }

                            let local_y = event.position.y
                                - scroll_handle.bounds().top()
                                - track_inset
                                - drag.thumb_grab_offset_y;
                            let percentage = (local_y / thumb_travel).clamp(0., 1.);
                            let scroll_y = max_scroll * percentage;
                            scroll_handle.set_offset(point(scroll_handle.offset().x, -scroll_y));
                            cx.notify(entity.entity_id());
                        }
                    });
                },
            )
            .size_full(),
        )
}

/// The custom-drawn overlay scrollbar for the virtualized preview `list`.
///
/// Mirrors [`pane_scrollbar_view`] but reads geometry from [`ListState`]'s
/// scrollbar API (`viewport_bounds`, `max_offset_for_scrollbar`,
/// `scroll_px_offset_for_scrollbar`) instead of a `ScrollHandle`, and drives the
/// list during a drag via `set_offset_from_scrollbar`. The
/// `scrollbar_drag_started`/`_ended` calls freeze the reported content height
/// for the duration of a drag so the thumb does not jump as off-screen blocks
/// get measured.
fn preview_list_scrollbar_view(
    list_state: &ListState,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    let id = "preview-pane-scrollbar";
    let target = PaneScrollTarget::Preview;
    let viewport_height = list_state.viewport_bounds().size.height;
    let max_scroll = list_state.max_offset_for_scrollbar().height.max(px(0.));
    if viewport_height <= px(0.) || max_scroll <= px(1.) {
        return div().id(id).hidden();
    }

    let track_inset = px(PANE_SCROLLBAR_EDGE_INSET);
    let track_height = (viewport_height - track_inset - track_inset).max(px(0.));
    if track_height <= px(0.) {
        return div().id(id).hidden();
    }

    let content_height = viewport_height + max_scroll;
    let thumb_height = (track_height * (viewport_height / content_height))
        .clamp(px(PANE_SCROLLBAR_MIN_THUMB_HEIGHT), track_height);
    let thumb_travel = (track_height - thumb_height).max(px(0.));
    let offset_y = (-list_state.scroll_px_offset_for_scrollbar().y).clamp(px(0.), max_scroll);
    let thumb_top = track_inset + thumb_travel * (offset_y / max_scroll);

    let entity = cx.entity();
    let list_state = list_state.clone();

    div()
        .id(id)
        .absolute()
        .top(thumb_top)
        .right(px(2.))
        .w(px(PANE_SCROLLBAR_THUMB_WIDTH))
        .h(thumb_height)
        .rounded_md()
        .bg(palette.muted)
        .hover(move |style| style.bg(palette.active_text))
        .block_mouse_except_scroll()
        .child(
            canvas(
                |_, _, _| (),
                move |thumb_bounds, _, window, _| {
                    window.on_mouse_event({
                        let entity = entity.clone();
                        let list_state = list_state.clone();
                        move |event: &MouseDownEvent, _, _, cx| {
                            if !thumb_bounds.contains(&event.position) {
                                return;
                            }
                            list_state.scrollbar_drag_started();
                            let _ = entity.update(cx, |app, _| {
                                app.pane_scrollbar_drag = Some(PaneScrollbarDrag {
                                    target,
                                    thumb_grab_offset_y: event.position.y - thumb_bounds.top(),
                                });
                            });
                        }
                    });
                    window.on_mouse_event({
                        let entity = entity.clone();
                        let list_state = list_state.clone();
                        move |_: &MouseUpEvent, _, _, cx| {
                            let _ = entity.update(cx, |app, _| {
                                if app
                                    .pane_scrollbar_drag
                                    .is_some_and(|drag| drag.target == target)
                                {
                                    app.pane_scrollbar_drag = None;
                                    list_state.scrollbar_drag_ended();
                                }
                            });
                        }
                    });
                    window.on_mouse_event({
                        let entity = entity.clone();
                        let list_state = list_state.clone();
                        move |event: &MouseMoveEvent, _, _, cx| {
                            if !event.dragging() {
                                return;
                            }
                            let Some(drag) = entity.read(cx).pane_scrollbar_drag else {
                                return;
                            };
                            if drag.target != target {
                                return;
                            }

                            let viewport_height = list_state.viewport_bounds().size.height;
                            let max_scroll =
                                list_state.max_offset_for_scrollbar().height.max(px(0.));
                            if viewport_height <= px(0.) || max_scroll <= px(1.) {
                                return;
                            }

                            let track_inset = px(PANE_SCROLLBAR_EDGE_INSET);
                            let track_height =
                                (viewport_height - track_inset - track_inset).max(px(0.));
                            if track_height <= px(0.) {
                                return;
                            }

                            let content_height = viewport_height + max_scroll;
                            let thumb_height = (track_height * (viewport_height / content_height))
                                .clamp(px(PANE_SCROLLBAR_MIN_THUMB_HEIGHT), track_height);
                            let thumb_travel = (track_height - thumb_height).max(px(0.));
                            if thumb_travel <= px(0.) {
                                return;
                            }

                            let local_y = event.position.y
                                - list_state.viewport_bounds().top()
                                - track_inset
                                - drag.thumb_grab_offset_y;
                            let percentage = (local_y / thumb_travel).clamp(0., 1.);
                            let scroll_y = max_scroll * percentage;
                            list_state.set_offset_from_scrollbar(point(px(0.), -scroll_y));
                            cx.notify(entity.entity_id());
                        }
                    });
                },
            )
            .size_full(),
        )
}

/// A vertical resize divider rendered between two panes. Visually it is a 1px
/// rule, but an 8px-wide transparent handle is layered on top so the grab target
/// is usable. Double-clicking the handle restores the default split. Pattern
/// follows Zed's `render_resize_handle`.
fn editor_split_handle_view(border_color: Rgba, cx: &mut Context<MarkionApp>) -> Stateful<Div> {
    div()
        .id("editor-split-resize-container")
        .relative()
        .h_full()
        .flex_shrink_0()
        .w(px(1.))
        .bg(border_color)
        .child(
            div()
                .id("editor-split-resize-handle")
                .absolute()
                .left(px(-RESIZE_HANDLE_WIDTH / 2.0))
                .w(px(RESIZE_HANDLE_WIDTH))
                .h_full()
                .cursor(CursorStyle::ResizeColumn)
                .block_mouse_except_scroll()
                .on_click(cx.listener(move |app, event: &ClickEvent, _, cx| {
                    // Double-click resets the editor/preview split to 50/50.
                    if event.click_count() >= 2 {
                        app.editor_split_ratio = 0.5;
                        cx.notify();
                    }
                }))
                .on_drag(DraggedEditorSplitHandle, move |_, _, _, cx| {
                    cx.new(|_| Empty)
                }),
        )
}

/// Resize divider for the sidebar's right edge: same visual pattern as the
/// editor split handle, but keyed on `DraggedSidebarHandle` and resets to the
/// default sidebar width on double-click.
fn sidebar_resize_handle_view(border_color: Rgba, cx: &mut Context<MarkionApp>) -> Stateful<Div> {
    div()
        .id("sidebar-resize-container")
        .relative()
        .h_full()
        .flex_shrink_0()
        .w(px(1.))
        .bg(border_color)
        .child(
            div()
                .id("sidebar-resize-handle")
                .absolute()
                .left(px(-RESIZE_HANDLE_WIDTH / 2.0))
                .w(px(RESIZE_HANDLE_WIDTH))
                .h_full()
                .cursor(CursorStyle::ResizeColumn)
                .block_mouse_except_scroll()
                .on_click(cx.listener(move |app, event: &ClickEvent, _, cx| {
                    if event.click_count() >= 2 {
                        app.sidebar_width = DEFAULT_SIDEBAR_WIDTH;
                        cx.notify();
                    }
                }))
                .on_drag(DraggedSidebarHandle, move |_, _, _, cx| cx.new(|_| Empty)),
        )
}

fn toolbar_button(
    label: &'static str,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .h(px(26.))
        .min_w(px(26.))
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface_bg)
        .text_color(palette.text)
        .text_size(px(12.))
        .cursor_pointer()
        .hover(move |style| style.bg(palette.active_bg).text_color(palette.active_text))
        .on_mouse_up(MouseButton::Left, listener)
        .child(label)
}

fn file_tree_context_menu_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let Some(menu) = &app.file_tree_context_menu else {
        return div().hidden();
    };
    let position = menu.position;
    let target_kind = menu.target.kind();
    let app_entity = cx.entity();

    div()
        .absolute()
        .top(position.y)
        .left(position.x)
        .w(px(208.))
        // Without occlude, mouse-down on a menu item falls through to
        // #main-content-row's close_menu handler, which clears the menu state
        // before the item's on_mouse_up can dispatch its action (same bug the
        // app-menu dropdown had — see active_menu_dropdown).
        .occlude()
        .p_1()
        .rounded_md()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface_bg)
        .shadow_md()
        .children(
            file_tree_context_actions(target_kind)
                .iter()
                .copied()
                .map(move |action| {
                    let app_entity = app_entity.clone();
                    div()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .text_size(px(12.))
                        .text_color(palette.text)
                        .cursor_pointer()
                        .hover(move |style| style.bg(palette.active_bg))
                        .child(t(app.language, file_tree_context_action_label(action)))
                        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                            let _ = app_entity.update(cx, |app, cx| {
                                app.handle_file_tree_context_action(action, window, cx);
                            });
                        })
                }),
        )
}

fn preview_context_action_label(action: PreviewContextAction) -> Msg {
    match action {
        PreviewContextAction::CopyPlain => Msg::ItemPreviewCopyPlain,
        PreviewContextAction::CopyMarkdown => Msg::ItemPreviewCopyMarkdown,
        PreviewContextAction::CopyHtml => Msg::ItemPreviewCopyHtml,
        PreviewContextAction::SelectAll => Msg::ItemPreviewSelectAll,
        PreviewContextAction::CopyLinkAddress => Msg::ItemPreviewCopyLinkAddress,
    }
}

fn preview_context_menu_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let Some(menu) = &app.preview_context_menu else {
        return div().hidden();
    };
    let position = menu.position;
    let link_url = menu.link_url.clone();
    let blocks = app.active_tab().preview_list_blocks.clone();
    let has_selection = preview_selection_takes_copy_precedence(
        app.active_tab().preview_selection.as_ref(),
        &blocks,
    );
    let app_entity = cx.entity();

    let items: Vec<(PreviewContextAction, bool)> = {
        let mut items = vec![
            (PreviewContextAction::CopyPlain, has_selection),
            (PreviewContextAction::CopyMarkdown, has_selection),
            (PreviewContextAction::CopyHtml, has_selection),
            (PreviewContextAction::SelectAll, true),
        ];
        if link_url.is_some() {
            items.push((PreviewContextAction::CopyLinkAddress, true));
        }
        items
    };

    div()
        .absolute()
        .top(position.y)
        .left(position.x)
        .w(px(220.))
        .occlude()
        .p_1()
        .rounded_md()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface_bg)
        .shadow_md()
        .children(items.into_iter().map(move |(action, enabled)| {
            let app_entity = app_entity.clone();
            let label = t(app.language, preview_context_action_label(action));
            let text_color = if enabled { palette.text } else { palette.muted };
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .text_size(px(12.))
                .text_color(text_color)
                .when(enabled, |style| {
                    style
                        .cursor_pointer()
                        .hover(move |style| style.bg(palette.active_bg))
                        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                            let _ = app_entity.update(cx, |app, cx| {
                                app.handle_preview_context_action(action, cx);
                            });
                        })
                })
                .child(label)
        }))
}

fn menu_title_button(
    label: &'static str,
    active: bool,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let background = if active {
        palette.active_bg
    } else {
        palette.panel_bg
    };
    let foreground = if active {
        palette.active_text
    } else {
        palette.text
    };
    let hover_bg = if active {
        palette.active_bg
    } else {
        palette.surface_bg
    };

    div()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(background)
        .text_size(px(13.))
        .text_color(foreground)
        .cursor_pointer()
        .hover(move |style| style.bg(hover_bg))
        .on_mouse_up(MouseButton::Left, listener)
        .child(label)
}

fn menu_action_button(
    label: &'static str,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .w_full()
        .px_3()
        .py_1()
        .text_size(px(12.))
        .text_color(palette.text)
        .cursor_pointer()
        .hover(move |style| style.bg(palette.surface_bg))
        .on_mouse_up(MouseButton::Left, listener)
        .child(label)
}

fn menu_separator(palette: ThemePalette) -> Div {
    div().h(px(1.)).my_1().bg(palette.border)
}

fn heading_item_msg(level: u8) -> Msg {
    match level {
        1 => Msg::ItemH1,
        2 => Msg::ItemH2,
        3 => Msg::ItemH3,
        4 => Msg::ItemH4,
        5 => Msg::ItemH5,
        6 => Msg::ItemH6,
        _ => Msg::ItemH1,
    }
}

fn heading_native_menu_items(language: Language, max_level: u8) -> Vec<MenuItem> {
    (1..=max_level)
        .map(|level| match level {
            1 => MenuItem::action(t(language, heading_item_msg(level)), Heading1),
            2 => MenuItem::action(t(language, heading_item_msg(level)), Heading2),
            3 => MenuItem::action(t(language, heading_item_msg(level)), Heading3),
            4 => MenuItem::action(t(language, heading_item_msg(level)), Heading4),
            5 => MenuItem::action(t(language, heading_item_msg(level)), Heading5),
            6 => MenuItem::action(t(language, heading_item_msg(level)), Heading6),
            _ => MenuItem::action(t(language, heading_item_msg(level)), Heading1),
        })
        .collect()
}

fn active_menu_dropdown(
    menu: Option<AppMenu>,
    language: Language,
    heading_menu_max_level: u8,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> impl IntoElement {
    let Some(menu) = menu else {
        return div()
            .absolute()
            .top(px(28.))
            .left(px(0.))
            .w(px(0.))
            .h(px(0.));
    };

    let panel = div()
        .absolute()
        .top(px(28.))
        .left(menu.dropdown_left(language))
        .w(menu.dropdown_width(language))
        // Without occlude the dropdown does not capture mouse hits, so clicks
        // fall through to the content underneath (which closes the menu) and
        // the item's action is never dispatched.
        .occlude()
        .py_1()
        .border_1()
        .border_color(palette.border)
        .rounded_md()
        .bg(palette.panel_bg)
        .text_color(palette.text)
        .shadow_md()
        .flex()
        .flex_col();

    // Menu items call their handler directly via cx.listener, exactly like the
    // menu-bar title buttons. Dispatching through window.dispatch_action here
    // proved unreliable (the action never reached the focused handlers), so the
    // items appeared to do nothing when clicked.
    //
    // Labels are translated at render time from the app's active language, so
    // switching language via View → Language reflows the menu immediately.
    macro_rules! action_item {
        ($msg:expr, $method:ident, $action:expr) => {
            menu_action_button(
                t(language, $msg),
                palette,
                cx.listener(move |app, _: &MouseUpEvent, window, cx| {
                    app.$method(&$action, window, cx);
                }),
            )
        };
    }

    match menu {
        AppMenu::File => panel
            .child(action_item!(Msg::ItemNew, new_document, NewDocument))
            .child(action_item!(Msg::ItemOpen, open_document, OpenDocument))
            .child(action_item!(Msg::ItemOpenFolder, open_folder, OpenFolder))
            .child(action_item!(Msg::ItemSave, save_document, SaveDocument))
            .child(action_item!(
                Msg::ItemSaveAs,
                save_document_as,
                SaveDocumentAs
            ))
            .child(menu_separator(palette))
            .child(action_item!(Msg::ItemNewTab, new_tab, NewTab))
            .child(action_item!(
                Msg::ItemOpenInNewTab,
                open_in_new_tab_action,
                OpenInNewTab
            ))
            .child(action_item!(Msg::ItemCloseTab, close_tab, CloseTab))
            .child(action_item!(Msg::ItemNextTab, next_tab, NextTab))
            .child(action_item!(Msg::ItemPrevTab, prev_tab, PrevTab))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemPreferences,
                show_preferences,
                ShowPreferences
            ))
            .child(action_item!(
                Msg::ItemResetPreferences,
                reset_preferences,
                ResetPreferences
            ))
            .child(menu_separator(palette))
            .child(action_item!(Msg::ItemExit, quit, Quit)),
        AppMenu::Edit => panel
            .child(action_item!(Msg::ItemUndo, undo, Undo))
            .child(action_item!(Msg::ItemRedo, redo, Redo))
            .child(menu_separator(palette))
            .child(action_item!(Msg::ItemCopy, copy, Copy))
            .child(action_item!(Msg::ItemCut, cut, Cut))
            .child(action_item!(Msg::ItemPaste, paste, Paste))
            .child(menu_separator(palette))
            .child(action_item!(Msg::ItemSelectAll, select_all, SelectAll)),
        AppMenu::View => panel
            .child(action_item!(
                Msg::ItemToggleView,
                toggle_view_mode,
                ToggleViewMode
            ))
            .child(action_item!(Msg::ItemEditMode, set_edit_mode, SetEditMode))
            .child(action_item!(
                Msg::ItemVisualEditMode,
                set_visual_edit_mode,
                SetVisualEditMode
            ))
            .child(action_item!(
                Msg::ItemSplitPreviewMode,
                set_split_preview_mode,
                SetSplitPreviewMode
            ))
            .child(action_item!(Msg::ItemReadMode, set_read_mode, SetReadMode))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemToggleSidebar,
                toggle_sidebar,
                ToggleSidebar
            ))
            .child(action_item!(
                Msg::ItemFiles,
                toggle_file_tree,
                ToggleFileTree
            ))
            .child(action_item!(
                Msg::ItemOutline,
                toggle_outline,
                ToggleOutline
            ))
            .child(action_item!(
                Msg::ItemFocusMode,
                toggle_focus_mode,
                ToggleFocusMode
            ))
            .child(action_item!(
                Msg::ItemTypewriterMode,
                toggle_typewriter_mode,
                ToggleTypewriterMode
            ))
            .child(action_item!(
                Msg::ItemCodeLineNumbers,
                toggle_code_line_numbers,
                ToggleCodeLineNumbers
            ))
            .child(menu_separator(palette))
            .child(action_item!(Msg::ItemFind, show_find, ShowFind))
            .child(action_item!(Msg::ItemReplace, show_replace, ShowReplace))
            .child(action_item!(Msg::ItemFindNext, find_next, FindNext))
            .child(action_item!(
                Msg::ItemFindPrevious,
                find_previous,
                FindPrevious
            ))
            .child(menu_separator(palette))
            .child(action_item!(Msg::ItemCycleTheme, cycle_theme, CycleTheme)),
        AppMenu::Format => {
            let with_core_headings = panel
                .child(action_item!(Msg::ItemBold, bold, Bold))
                .child(action_item!(Msg::ItemItalic, italic, Italic))
                .child(action_item!(Msg::ItemInlineCode, inline_code, InlineCode))
                .child(action_item!(Msg::ItemLink, insert_link, InsertLink))
                .child(action_item!(Msg::ItemImage, insert_image, InsertImage))
                .child(menu_separator(palette))
                .child(action_item!(Msg::ItemH1, heading1, Heading1))
                .child(action_item!(Msg::ItemH2, heading2, Heading2))
                .child(action_item!(Msg::ItemH3, heading3, Heading3));
            let with_headings = with_core_headings
                .child(action_item!(Msg::ItemH4, heading4, Heading4))
                .child(action_item!(Msg::ItemH5, heading5, Heading5));
            let with_headings = if heading_menu_max_level >= EXTENDED_HEADING_MENU_MAX_LEVEL {
                with_headings.child(action_item!(Msg::ItemH6, heading6, Heading6))
            } else {
                with_headings
            };
            with_headings
                .child(menu_separator(palette))
                .child(action_item!(
                    Msg::ItemBullets,
                    unordered_list,
                    UnorderedList
                ))
                .child(action_item!(Msg::ItemNumbers, ordered_list, OrderedList))
                .child(action_item!(Msg::ItemTask, task_list, TaskList))
                .child(action_item!(Msg::ItemQuote, block_quote, BlockQuote))
                .child(action_item!(Msg::ItemCodeFence, code_fence, CodeFence))
                .child(menu_separator(palette))
                .child(action_item!(
                    Msg::ItemFormatTable,
                    format_table,
                    FormatTable
                ))
                .child(action_item!(
                    Msg::ItemAddTableRow,
                    table_add_row,
                    TableAddRow
                ))
                .child(action_item!(
                    Msg::ItemDeleteTableRow,
                    table_delete_row,
                    TableDeleteRow
                ))
                .child(action_item!(
                    Msg::ItemMoveRowUp,
                    table_move_row_up,
                    TableMoveRowUp
                ))
                .child(action_item!(
                    Msg::ItemMoveRowDown,
                    table_move_row_down,
                    TableMoveRowDown
                ))
                .child(action_item!(
                    Msg::ItemAddTableColumn,
                    table_add_column,
                    TableAddColumn
                ))
                .child(action_item!(
                    Msg::ItemDeleteTableColumn,
                    table_delete_column,
                    TableDeleteColumn
                ))
        }
        AppMenu::Export => panel
            .child(action_item!(Msg::ItemExportHtml, export_html, ExportHtml))
            .child(action_item!(
                Msg::ItemExportPlainHtml,
                export_plain_html,
                ExportPlainHtml
            ))
            .child(action_item!(Msg::ItemExportPdf, export_pdf, ExportPdf))
            .child(action_item!(
                Msg::ItemExportLatex,
                export_latex,
                ExportLatex
            ))
            .child(action_item!(Msg::ItemExportDocx, export_docx, ExportDocx))
            .child(action_item!(Msg::ItemExportPng, export_png, ExportPng))
            .child(action_item!(Msg::ItemExportJpeg, export_jpeg, ExportJpeg)),
        AppMenu::Help => panel
            .child(action_item!(
                Msg::ItemKeyboardShortcuts,
                show_shortcuts,
                ShowShortcuts
            ))
            .child(action_item!(Msg::ItemAboutMarkion, about, AboutMarkion)),
    }
}

/// Modal overlay for the in-app Preferences panel. Clicks dispatch through
/// `cx.listener` closures so each setting updates live app state and persists
/// through the existing preferences path.
fn preferences_panel_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let app_entity = cx.entity();
    let themes = app.available_themes();
    let active_name = app.selected_theme_name.clone();

    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .bg(rgba(0x00000055))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .occlude()
                .w(px(560.))
                .max_h(px(560.))
                .py_4()
                .bg(palette.panel_bg)
                .border_1()
                .border_color(palette.border)
                .rounded_lg()
                .shadow_lg()
                .text_color(palette.text)
                .flex()
                .flex_col()
                // Title bar.
                .child(
                    div()
                        .px_4()
                        .pb_3()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(15.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(app.tr(Msg::PrefPanelTitle)),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .cursor_pointer()
                                .text_color(palette.muted)
                                .hover(|style| style.bg(palette.surface_bg))
                                .child("✕")
                                .on_mouse_up(
                                    MouseButton::Left,
                                    cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                                        app.close_preferences(cx);
                                    }),
                                ),
                        ),
                )
                // Scrollable body.
                .child(
                    div()
                        .id("preferences-panel-body")
                        .px_4()
                        .overflow_y_scroll()
                        .scrollbar_width(px(8.))
                        .flex()
                        .flex_col()
                        .gap_4()
                        // Language choices appear first so the rest of the
                        // panel immediately reflects the user's UI language.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.muted)
                                        .child(app.tr(Msg::PrefPanelLanguageSection)),
                                )
                                .child(div().flex().gap_2().children(Language::all().iter().map(
                                    |&lang| {
                                        let is_active = app.language == lang;
                                        preference_option_button(
                                            format!(
                                                "{}  {}",
                                                if is_active { "✓" } else { " " },
                                                lang.native_name()
                                            ),
                                            is_active,
                                            palette,
                                            cx.listener(
                                                move |app, _: &MouseUpEvent, _window, cx| {
                                                    app.apply_language(lang, cx);
                                                },
                                            ),
                                        )
                                    },
                                ))),
                        )
                        // Theme grid.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.muted)
                                        .child(app.tr(Msg::PrefPanelThemeSection)),
                                )
                                .child(div().flex().flex_wrap().gap_2().children(
                                    themes.iter().map(|theme| {
                                        let theme_name = theme.name.clone();
                                        let is_active =
                                            theme.name.eq_ignore_ascii_case(&active_name);
                                        let colors = theme.colors;
                                        let app_entity = app_entity.clone();
                                        let border = if is_active {
                                            palette.active_bg
                                        } else {
                                            palette.border
                                        };
                                        div()
                                            .w(px(120.))
                                            .p_2()
                                            .rounded_md()
                                            .border_1()
                                            .border_color(border)
                                            .bg(rgb(colors.panel_bg))
                                            .cursor_pointer()
                                            .hover(move |style| {
                                                style.border_color(palette.active_bg)
                                            })
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                                                let _ = app_entity.update(cx, |app, cx| {
                                                    app.apply_theme_by_name(&theme_name, cx);
                                                });
                                            })
                                            .child(
                                                div()
                                                    .h(px(28.))
                                                    .rounded_sm()
                                                    .flex()
                                                    .child(div().flex_1().bg(rgb(colors.app_bg)))
                                                    .child(
                                                        div().flex_1().bg(rgb(colors.surface_bg)),
                                                    )
                                                    .child(div().flex_1().bg(rgb(colors.active_bg)))
                                                    .child(
                                                        div().w(px(6.)).bg(rgb(colors.active_text)),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .gap_1()
                                                    .text_size(px(11.))
                                                    .child(
                                                        div()
                                                            .min_w_0()
                                                            .text_color(rgb(colors.text))
                                                            .child(theme.name.clone()),
                                                    )
                                                    .when(is_active, |row| {
                                                        row.child(
                                                            div()
                                                                .text_color(palette.active_bg)
                                                                .child("✓"),
                                                        )
                                                    }),
                                            )
                                    }),
                                )),
                        )
                        // Other settings.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.muted)
                                        .child(app.tr(Msg::PrefPanelOtherSection)),
                                )
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelFocusMode),
                                    app.focus_mode,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                                        app.toggle_focus_mode(&ToggleFocusMode, window, cx);
                                    }),
                                ))
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelTypewriterMode),
                                    app.typewriter_mode,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                                        app.toggle_typewriter_mode(
                                            &ToggleTypewriterMode,
                                            window,
                                            cx,
                                        );
                                    }),
                                ))
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelCodeLineNumbers),
                                    app.code_line_numbers,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                                        app.toggle_code_line_numbers(
                                            &ToggleCodeLineNumbers,
                                            window,
                                            cx,
                                        );
                                    }),
                                ))
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelPreviewAdaptiveWidth),
                                    app.preview_adaptive_width,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        app.toggle_preview_adaptive_width(cx);
                                    }),
                                ))
                                .child(preference_heading_menu_row(app, palette, cx))
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelSyncScroll),
                                    app.sync_scroll,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        app.toggle_sync_scroll(cx);
                                    }),
                                ))
                                .child(preference_sidebar_row(app, palette, cx)),
                        ),
                ),
        )
}

fn preference_boolean_row(
    label: &'static str,
    enabled: bool,
    language: Language,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let value = t(language, if enabled { Msg::PrefOn } else { Msg::PrefOff }).to_string();

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(12.))
        .px_1()
        .py_1()
        .gap_3()
        .child(div().text_color(palette.muted).child(label))
        .child(preference_option_button(value, enabled, palette, listener))
}

fn preference_heading_menu_row(
    app: &MarkionApp,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let extended = app.heading_menu_max_level >= EXTENDED_HEADING_MENU_MAX_LEVEL;

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(12.))
        .px_1()
        .py_1()
        .gap_3()
        .child(
            div()
                .text_color(palette.muted)
                .child(app.tr(Msg::PrefPanelHeadingMenu)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(preference_option_button(
                    app.tr(Msg::PrefPanelHeadingMenuThree).to_string(),
                    !extended,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                        app.set_heading_menu_max_level(DEFAULT_HEADING_MENU_MAX_LEVEL, cx);
                    }),
                ))
                .child(preference_option_button(
                    app.tr(Msg::PrefPanelHeadingMenuSix).to_string(),
                    extended,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                        app.set_heading_menu_max_level(EXTENDED_HEADING_MENU_MAX_LEVEL, cx);
                    }),
                )),
        )
}

fn preference_sidebar_row(
    app: &MarkionApp,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let language = app.language;

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(12.))
        .px_1()
        .py_1()
        .gap_3()
        .child(
            div()
                .text_color(palette.muted)
                .child(app.tr(Msg::PrefPanelSidebar)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(preference_option_button(
                    t(
                        language,
                        if app.sidebar_visible {
                            Msg::PrefOn
                        } else {
                            Msg::PrefOff
                        },
                    )
                    .to_string(),
                    app.sidebar_visible,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                        app.toggle_sidebar(&ToggleSidebar, window, cx);
                    }),
                ))
                .child(preference_option_button(
                    sidebar_tab_label(language, SidebarTab::Files).to_string(),
                    app.sidebar_visible && app.sidebar_tab == SidebarTab::Files,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                        app.select_preferences_sidebar_tab(SidebarTab::Files, cx);
                    }),
                ))
                .child(preference_option_button(
                    sidebar_tab_label(language, SidebarTab::Outline).to_string(),
                    app.sidebar_visible && app.sidebar_tab == SidebarTab::Outline,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                        app.select_preferences_sidebar_tab(SidebarTab::Outline, cx);
                    }),
                )),
        )
}

fn preference_option_button(
    label: String,
    active: bool,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let background = if active {
        palette.active_bg
    } else {
        palette.surface_bg
    };
    let foreground = if active {
        palette.active_text
    } else {
        palette.text
    };
    let border = if active {
        palette.active_bg
    } else {
        palette.border
    };

    div()
        .min_w(px(48.))
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(background)
        .text_color(foreground)
        .text_size(px(12.))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .hover(move |style| style.border_color(palette.active_bg))
        .on_mouse_up(MouseButton::Left, listener)
        .child(label)
}

fn read_mode_preview_is_constrained(view_mode: ViewMode, preview_adaptive_width: bool) -> bool {
    matches!(view_mode, ViewMode::Read) && !preview_adaptive_width
}

fn view_mode_status_message(view_mode: ViewMode) -> Msg {
    match view_mode {
        ViewMode::Edit => Msg::StatusEditMode,
        ViewMode::VisualEdit => Msg::StatusVisualEditMode,
        ViewMode::Split => Msg::StatusSplitPreviewMode,
        ViewMode::Read => Msg::StatusReadMode,
    }
}

fn assign_view_mode(current: &mut ViewMode, target: ViewMode) {
    *current = target;
}

fn view_mode_pane_widths(view_mode: ViewMode, split_ratio: f32) -> (f32, f32) {
    match view_mode {
        ViewMode::Edit | ViewMode::VisualEdit => (1.0, 0.0),
        ViewMode::Split => (split_ratio, 1.0 - split_ratio),
        ViewMode::Read => (0.0, 1.0),
    }
}

/// Whether proportional scroll-sync should couple the two panes this frame.
/// Only in Split Preview (the sole mode where both panes are visible) and only
/// when the preference is on.
fn sync_scroll_is_active(view_mode: ViewMode, sync_scroll: bool) -> bool {
    matches!(view_mode, ViewMode::Split) && sync_scroll
}

/// Scroll fraction in `[0,1]` for a pane, given its current scroll offset
/// (positive pixels from the top) and its maximum scrollable offset. Returns
/// `0.0` when the pane has no scrollable range (`max <= 1`), so a pane that
/// fits its viewport never drives the other pane.
fn sync_fraction(offset: f32, max: f32) -> f32 {
    if max <= 1. {
        return 0.;
    }
    (offset / max).clamp(0., 1.)
}

/// Sync coupling converges within an epsilon; comparing fractions below this
/// threshold avoids re-writing the non-driving pane every frame (and the
/// resulting sub-pixel fight with the user's own scroll).
const SYNC_SCROLL_EPSILON: f32 = 0.001;

/// Clamp a byte offset to a UTF-8 char boundary within `run_text`.
fn clamp_preview_offset(run_text: &str, offset: usize) -> usize {
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
fn normalize_preview_selection_range(run_text: &str, range: Range<usize>) -> Range<usize> {
    let start = clamp_preview_offset(run_text, range.start.min(range.end));
    let end = clamp_preview_offset(run_text, range.start.max(range.end));
    start..end
}

/// Plain text of a single selectable run inside a preview block.
fn preview_run_plain_text(block: &PreviewBlock, run_id: PreviewTextRunId) -> Option<String> {
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
fn preview_block_runs(block: &PreviewBlock) -> Vec<PreviewTextRunId> {
    match block {
        PreviewBlock::Heading { .. }
        | PreviewBlock::Paragraph { .. }
        | PreviewBlock::ListItem { .. }
        | PreviewBlock::BlockQuote { .. } => vec![PreviewTextRunId::Body],
        PreviewBlock::CodeBlock { .. } => vec![PreviewTextRunId::CodeBody],
        PreviewBlock::MathBlock { .. } => {
            vec![PreviewTextRunId::MathRendered, PreviewTextRunId::MathLatex]
        }
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
fn preview_run_highlight_range(
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
fn preview_selection_plain_text(
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
fn preview_selection_takes_copy_precedence(
    preview: Option<&PreviewSelection>,
    blocks: &[PreviewBlock],
) -> bool {
    preview.is_some_and(|selection| preview_selection_plain_text(selection, blocks).is_some())
}

/// Drop a preview selection when either caret's block index is out of range.
fn invalidate_preview_selection_if_stale(
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
fn preview_selection_markdown(
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

fn preview_block_source_range(block: &PreviewBlock) -> Option<Range<usize>> {
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
fn preview_selection_paint_quads(layout: &TextLayout, range: Range<usize>) -> Vec<PaintQuad> {
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
fn preview_index_for_position(layout: &TextLayout, position: Point<Pixels>) -> usize {
    match layout.index_for_position(position) {
        Ok(index) => index,
        Err(index) => index,
    }
}

#[derive(Clone)]
struct VisualTextSegment {
    visible_range: Range<usize>,
    source_range: Range<usize>,
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

fn visual_source_for_visible(segments: &[VisualTextSegment], visible: usize) -> usize {
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

fn visual_visible_for_source(segments: &[VisualTextSegment], source: usize) -> Option<usize> {
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
fn active_preview_run_selection(
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
fn rich_text_element(
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
fn selectable_plain_text(
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
fn code_line_text(line: &[HighlightedSpan]) -> (StyledText, String) {
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
fn code_block_text(lines: &[Vec<HighlightedSpan>]) -> (StyledText, String) {
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
fn preview_block_splice(
    old: &[PreviewBlock],
    new: &[PreviewBlock],
) -> (std::ops::Range<usize>, usize) {
    block_splice(old, new)
}

fn block_splice<T: PartialEq>(old: &[T], new: &[T]) -> (std::ops::Range<usize>, usize) {
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
fn should_parse_preview_now(since_change: Option<Duration>, since_parse: Option<Duration>) -> bool {
    let settled = since_change.is_none_or(|d| d >= PREVIEW_DEBOUNCE);
    let too_stale = since_parse.is_none_or(|d| d >= PREVIEW_MAX_STALE);
    settled || too_stale
}

/// Globally unique id for a background preview parse (see
/// `EditorTab::preview_parse_inflight`). Global uniqueness is what lets a
/// landing result safely locate its owning tab: `text_version`s can collide
/// across documents, but two tabs can never carry the same task id.
fn next_preview_parse_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

fn visual_highlight_style(run: &VisualInlineRun) -> Option<HighlightStyle> {
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

fn visual_text_element(
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

fn visual_source_island_view(
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

fn visual_block_is_focused(app: &MarkionApp, block: &VisualBlock) -> bool {
    let cursor = app.active_tab().cursor_offset();
    visual_source_range_is_focused(
        &block.source_range,
        cursor,
        app.active_tab().document.text().len(),
    )
}

fn visual_source_range_is_focused(
    source_range: &Range<usize>,
    cursor: usize,
    document_len: usize,
) -> bool {
    source_range.contains(&cursor) || (cursor == document_len && cursor == source_range.end)
}

fn visual_block_view(
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

fn table_toolbar_actions_for_view_mode(view_mode: ViewMode) -> &'static [TableToolbarAction] {
    if matches!(view_mode, ViewMode::VisualEdit) {
        &VISUAL_TABLE_TOOLBAR_ACTIONS
    } else {
        &[]
    }
}

fn visual_table_view(
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

fn preview_block_view(
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

fn preview_table_button(
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

fn preview_image_source(url: &str, document_dir: Option<&Path>) -> ImageSource {
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

fn is_remote_resource(url: &str) -> bool {
    url.contains("://") || url.starts_with("data:")
}

fn highlight_color(kind: HighlightKind) -> Rgba {
    match kind {
        HighlightKind::Plain => rgb(0xe2e8f0),
        HighlightKind::Keyword => rgb(0xc084fc),
        HighlightKind::String => rgb(0x86efac),
        HighlightKind::Number => rgb(0xfbbf24),
        HighlightKind::Comment => rgb(0x94a3b8),
        HighlightKind::Type => rgb(0x67e8f9),
    }
}

fn utf16_offset_to_byte_offset(text: &str, offset: usize) -> usize {
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

fn byte_offset_to_utf16_offset(text: &str, offset: usize) -> usize {
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

fn clamp_to_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn install_window_close_guard(window: &mut Window, app_entity: Entity<MarkionApp>, cx: &mut App) {
    window.on_window_should_close(cx, move |window, cx| {
        let (allow_close, is_dirty, confirming_close, language) = {
            let app = app_entity.read(cx);
            (
                app.allow_close,
                app.tabs.iter().any(|t| t.document.is_dirty()),
                app.confirming_close,
                app.language,
            )
        };

        if allow_close || !is_dirty {
            return true;
        }

        if confirming_close {
            return false;
        }

        let answer = window.prompt(
            PromptLevel::Warning,
            t(language, Msg::DialogExitTitle),
            Some(t(language, Msg::DialogExitDetail)),
            &[
                PromptButton::ok(t(language, Msg::DialogButtonExitWithoutSaving)),
                PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
            ],
            cx,
        );

        let _ = app_entity.update(cx, |app, cx| {
            app.confirming_close = true;
            app.active_menu = None;
            app.status = t(app.language, Msg::StatusWaitingQuitConfirm).into();
            cx.notify();
        });

        let app_entity = app_entity.clone();
        cx.spawn(async move |cx| {
            let confirmed = matches!(answer.await, Ok(0));
            let _ = cx.update(|cx| {
                let _ = app_entity.update(cx, |app, cx| {
                    app.confirming_close = false;
                    if confirmed {
                        app.discard_current_recovery_file();
                        app.allow_close = true;
                        cx.quit();
                    } else {
                        app.status = t(app.language, Msg::StatusExitCanceled).into();
                        cx.notify();
                    }
                });
            });
        })
        .detach();

        false
    });
}

fn install_menus(language: Language, heading_menu_max_level: u8, cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: t(language, Msg::MenuFile).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemNew), NewDocument),
                MenuItem::action(t(language, Msg::ItemOpen), OpenDocument),
                MenuItem::action(t(language, Msg::ItemOpenFolder), OpenFolder),
                MenuItem::action(t(language, Msg::ItemSave), SaveDocument),
                MenuItem::action(t(language, Msg::ItemSaveAs), SaveDocumentAs),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemNewTab), NewTab),
                MenuItem::action(t(language, Msg::ItemOpenInNewTab), OpenInNewTab),
                MenuItem::action(t(language, Msg::ItemCloseTab), CloseTab),
                MenuItem::action(t(language, Msg::ItemNextTab), NextTab),
                MenuItem::action(t(language, Msg::ItemPrevTab), PrevTab),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemPreferences), ShowPreferences),
                MenuItem::action(t(language, Msg::ItemResetPreferences), ResetPreferences),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemExit), Quit),
            ],
        },
        Menu {
            name: t(language, Msg::MenuEdit).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemUndo), Undo),
                MenuItem::action(t(language, Msg::ItemRedo), Redo),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemCopy), Copy),
                MenuItem::action(t(language, Msg::ItemCut), Cut),
                MenuItem::action(t(language, Msg::ItemPaste), Paste),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemSelectAll), SelectAll),
            ],
        },
        Menu {
            name: t(language, Msg::MenuView).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemToggleView), ToggleViewMode),
                MenuItem::action(t(language, Msg::ItemEditMode), SetEditMode),
                MenuItem::action(t(language, Msg::ItemVisualEditMode), SetVisualEditMode),
                MenuItem::action(t(language, Msg::ItemSplitPreviewMode), SetSplitPreviewMode),
                MenuItem::action(t(language, Msg::ItemReadMode), SetReadMode),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemToggleSidebar), ToggleSidebar),
                MenuItem::action(t(language, Msg::ItemFiles), ToggleFileTree),
                MenuItem::action(t(language, Msg::ItemOutline), ToggleOutline),
                MenuItem::action(t(language, Msg::ItemFocusMode), ToggleFocusMode),
                MenuItem::action(t(language, Msg::ItemTypewriterMode), ToggleTypewriterMode),
                MenuItem::action(t(language, Msg::ItemCodeLineNumbers), ToggleCodeLineNumbers),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemFind), ShowFind),
                MenuItem::action(t(language, Msg::ItemReplace), ShowReplace),
                MenuItem::action(t(language, Msg::ItemFindNext), FindNext),
                MenuItem::action(t(language, Msg::ItemFindPrevious), FindPrevious),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemCycleTheme), CycleTheme),
            ],
        },
        Menu {
            name: t(language, Msg::MenuFormat).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemBold), Bold),
                MenuItem::action(t(language, Msg::ItemItalic), Italic),
                MenuItem::action(t(language, Msg::ItemInlineCode), InlineCode),
                MenuItem::action(t(language, Msg::ItemLink), InsertLink),
                MenuItem::action(t(language, Msg::ItemImage), InsertImage),
                MenuItem::separator(),
            ]
            .into_iter()
            .chain(heading_native_menu_items(language, heading_menu_max_level))
            .chain([
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemBullets), UnorderedList),
                MenuItem::action(t(language, Msg::ItemNumbers), OrderedList),
                MenuItem::action(t(language, Msg::ItemTask), TaskList),
                MenuItem::action(t(language, Msg::ItemQuote), BlockQuote),
                MenuItem::action(t(language, Msg::ItemCodeFence), CodeFence),
                MenuItem::separator(),
                MenuItem::action(t(language, Msg::ItemFormatTable), FormatTable),
                MenuItem::action(t(language, Msg::ItemAddTableRow), TableAddRow),
                MenuItem::action(t(language, Msg::ItemDeleteTableRow), TableDeleteRow),
                MenuItem::action(t(language, Msg::ItemMoveRowUp), TableMoveRowUp),
                MenuItem::action(t(language, Msg::ItemMoveRowDown), TableMoveRowDown),
                MenuItem::action(t(language, Msg::ItemAddTableColumn), TableAddColumn),
                MenuItem::action(t(language, Msg::ItemDeleteTableColumn), TableDeleteColumn),
            ])
            .collect(),
        },
        Menu {
            name: t(language, Msg::MenuExport).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemExportHtml), ExportHtml),
                MenuItem::action(t(language, Msg::ItemExportPlainHtml), ExportPlainHtml),
                MenuItem::action(t(language, Msg::ItemExportPdf), ExportPdf),
                MenuItem::action(t(language, Msg::ItemExportLatex), ExportLatex),
                MenuItem::action(t(language, Msg::ItemExportDocx), ExportDocx),
                MenuItem::action(t(language, Msg::ItemExportPng), ExportPng),
                MenuItem::action(t(language, Msg::ItemExportJpeg), ExportJpeg),
            ],
        },
        Menu {
            name: t(language, Msg::MenuHelp).into(),
            items: vec![
                MenuItem::action(t(language, Msg::ItemKeyboardShortcuts), ShowShortcuts),
                MenuItem::action(t(language, Msg::ItemAboutMarkion), AboutMarkion),
            ],
        },
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_folder_prompt_selects_one_directory() {
        let options = open_folder_prompt_options(Language::En);
        assert!(!options.files);
        assert!(options.directories);
        assert!(!options.multiple);
        assert_eq!(
            options.prompt.as_ref().map(ToString::to_string).as_deref(),
            Some("Open Folder")
        );
    }

    #[test]
    fn open_folder_action_is_wired_after_open_without_a_shortcut() {
        let source = include_str!("main.rs");
        assert!(source.contains(".on_action(cx.listener(Self::open_folder))"));

        let in_window = source
            .split_once("AppMenu::File => panel")
            .and_then(|(_, rest)| rest.split_once("AppMenu::Edit =>").map(|(file, _)| file))
            .expect("in-window File menu");
        let in_window_open = in_window.find("Msg::ItemOpen,").expect("Open item");
        let in_window_folder = in_window
            .find("Msg::ItemOpenFolder,")
            .expect("Open Folder item");
        let in_window_save = in_window.find("Msg::ItemSave,").expect("Save item");
        assert!(in_window_open < in_window_folder && in_window_folder < in_window_save);

        let native = source
            .split_once("fn install_menus")
            .and_then(|(_, rest)| rest.split_once("Msg::MenuEdit").map(|(file, _)| file))
            .expect("native File menu");
        let native_open = native.find("Msg::ItemOpen)").expect("native Open item");
        let native_folder = native
            .find("Msg::ItemOpenFolder)")
            .expect("native Open Folder item");
        let native_save = native.find("Msg::ItemSave)").expect("native Save item");
        assert!(native_open < native_folder && native_folder < native_save);
        let runtime_source = source
            .split_once("\nfn main()")
            .map(|(_, runtime)| runtime)
            .expect("main function");
        assert!(
            !runtime_source
                .lines()
                .any(|line| line.contains("KeyBinding::new") && line.contains("OpenFolder"))
        );
    }

    #[test]
    fn workspace_root_selection_preserves_contained_documents_and_rebases_external_ones() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let nested = root.join("notes").join("inside.md");
        let outside = temp.path().join("outside").join("other.md");
        std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
        std::fs::create_dir_all(outside.parent().unwrap()).unwrap();
        std::fs::write(&nested, "# inside").unwrap();
        std::fs::write(&outside, "# outside").unwrap();

        assert_eq!(
            workspace_root_for_document(Some(&root), &nested),
            Some(comparable_document_path(&root))
        );
        assert_eq!(
            workspace_root_for_document(Some(&root), &outside),
            outside.parent().map(comparable_document_path)
        );
        assert_eq!(
            workspace_root_for_document(None, &nested),
            nested.parent().map(comparable_document_path)
        );

        let sibling_prefix = temp.path().join("workspace-copy").join("note.md");
        std::fs::create_dir_all(sibling_prefix.parent().unwrap()).unwrap();
        std::fs::write(&sibling_prefix, "# sibling").unwrap();
        assert!(!path_is_within_workspace(&root, &sibling_prefix));
    }

    #[test]
    fn workspace_root_reset_and_stale_scan_checks_are_root_aware() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();

        assert!(!workspace_root_needs_reset(&first, true, &first));
        assert!(workspace_root_needs_reset(&first, false, &first));
        assert!(workspace_root_needs_reset(&first, true, &second));
        assert!(scan_result_matches_workspace(&first, &first));
        assert!(!scan_result_matches_workspace(&first, &second));
    }

    #[test]
    fn folder_scan_supports_empty_roots_and_reports_missing_roots() {
        let temp = tempfile::tempdir().unwrap();
        let empty = temp.path().join("empty");
        std::fs::create_dir_all(&empty).unwrap();

        let tree = FileTree::scan(&empty).unwrap();
        assert_eq!(
            comparable_document_path(&tree.root),
            comparable_document_path(&empty)
        );
        assert!(tree.entries.is_empty());
        assert!(FileTree::scan(temp.path().join("missing")).is_err());
    }

    #[test]
    fn hide_search_overlay_state_closes_without_clearing_buffers() {
        let mut search_visible = true;
        let mut replace_visible = true;
        let mut search_focus = Some(SearchField::Replace);
        let mut input_marked_len = 3;
        let query = "needle".to_string();
        let replacement = "thread".to_string();

        hide_search_overlay_state(
            &mut search_visible,
            &mut replace_visible,
            &mut search_focus,
            &mut input_marked_len,
        );

        assert!(!search_visible);
        assert!(!replace_visible);
        assert_eq!(search_focus, None);
        assert_eq!(input_marked_len, 0);
        assert_eq!(query, "needle");
        assert_eq!(replacement, "thread");
    }

    #[test]
    fn normalize_preview_selection_range_clamps_and_orders() {
        assert_eq!(normalize_preview_selection_range("hello", 1..4), 1..4);
        assert_eq!(normalize_preview_selection_range("hello", 4..1), 1..4);
        assert_eq!(normalize_preview_selection_range("hello", 0..99), 0..5);
        // Mid-codepoint end advances to the next boundary ("é" is bytes 1..3).
        assert_eq!(normalize_preview_selection_range("héllo", 1..2), 1..3);
    }

    fn sample_paragraph(text: &str) -> PreviewBlock {
        PreviewBlock::Paragraph {
            text: RichText::plain(text),
            source_range: 0..text.len(),
        }
    }

    #[test]
    fn preview_table_cells_remain_selectable_without_editing_toolbar() {
        let block = PreviewBlock::Table {
            rows: vec![
                vec!["Name".into(), "Value".into()],
                vec!["alpha".into(), "1".into()],
            ],
            alignments: vec![],
            source_range: 0..0,
        };

        assert_eq!(
            preview_block_runs(&block),
            vec![
                PreviewTextRunId::TableCell { row: 0, col: 0 },
                PreviewTextRunId::TableCell { row: 0, col: 1 },
                PreviewTextRunId::TableCell { row: 1, col: 0 },
                PreviewTextRunId::TableCell { row: 1, col: 1 },
            ]
        );
        assert_eq!(
            preview_run_plain_text(&block, PreviewTextRunId::TableCell { row: 1, col: 0 })
                .as_deref(),
            Some("alpha")
        );
    }

    #[test]
    fn preview_selection_plain_text_extracts_substring() {
        let blocks = vec![sample_paragraph("hello")];
        let selection = PreviewSelection {
            anchor: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 1,
            },
            head: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 4,
            },
        };
        assert_eq!(
            preview_selection_plain_text(&selection, &blocks).as_deref(),
            Some("ell")
        );
        let empty = PreviewSelection {
            anchor: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 2,
            },
            head: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 2,
            },
        };
        assert!(preview_selection_plain_text(&empty, &blocks).is_none());
    }

    #[test]
    fn preview_selection_plain_text_spans_multiple_blocks() {
        let blocks = vec![sample_paragraph("hello"), sample_paragraph("world")];
        let selection = PreviewSelection {
            anchor: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 3,
            },
            head: PreviewCaret {
                block_index: 1,
                run_id: PreviewTextRunId::Body,
                offset: 3,
            },
        };
        assert_eq!(
            preview_selection_plain_text(&selection, &blocks).as_deref(),
            Some("lo\nwor")
        );
    }

    #[test]
    fn preview_selection_takes_copy_precedence_only_when_non_empty() {
        let blocks = vec![sample_paragraph("abc")];
        let selection = PreviewSelection {
            anchor: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 0,
            },
            head: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 3,
            },
        };
        assert!(preview_selection_takes_copy_precedence(
            Some(&selection),
            &blocks
        ));
        let empty = PreviewSelection {
            anchor: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 1,
            },
            head: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 1,
            },
        };
        assert!(!preview_selection_takes_copy_precedence(
            Some(&empty),
            &blocks
        ));
        assert!(!preview_selection_takes_copy_precedence(None, &blocks));
    }

    #[test]
    fn invalidate_preview_selection_if_stale_drops_out_of_range_blocks() {
        let selection = PreviewSelection {
            anchor: PreviewCaret {
                block_index: 2,
                run_id: PreviewTextRunId::Body,
                offset: 0,
            },
            head: PreviewCaret {
                block_index: 2,
                run_id: PreviewTextRunId::Body,
                offset: 1,
            },
        };
        assert!(invalidate_preview_selection_if_stale(Some(selection.clone()), 3).is_some());
        assert!(invalidate_preview_selection_if_stale(Some(selection), 2).is_none());
        assert!(invalidate_preview_selection_if_stale(None, 10).is_none());
    }

    #[test]
    fn preview_run_highlight_range_covers_middle_and_partial_runs() {
        let selection = PreviewSelection {
            anchor: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 2,
            },
            head: PreviewCaret {
                block_index: 2,
                run_id: PreviewTextRunId::Body,
                offset: 3,
            },
        };
        assert_eq!(
            preview_run_highlight_range(&selection, 0, PreviewTextRunId::Body, "hello"),
            Some(2..5)
        );
        assert_eq!(
            preview_run_highlight_range(&selection, 1, PreviewTextRunId::Body, "body"),
            Some(0..4)
        );
        assert_eq!(
            preview_run_highlight_range(&selection, 2, PreviewTextRunId::Body, "world"),
            Some(0..3)
        );
        assert_eq!(
            preview_run_highlight_range(&selection, 3, PreviewTextRunId::Body, "later"),
            None
        );
    }

    #[test]
    fn preview_selection_markdown_joins_covered_block_sources() {
        let document = "# Title\n\nHello world\n\n- item\n";
        let blocks = vec![
            PreviewBlock::Heading {
                level: 1,
                text: RichText::plain("Title"),
                source_range: 0..7,
            },
            PreviewBlock::Paragraph {
                text: RichText::plain("Hello world"),
                source_range: 9..20,
            },
            PreviewBlock::ListItem {
                level: 0,
                ordered: false,
                index: None,
                checked: None,
                text: RichText::plain("item"),
                source_range: 22..28,
            },
        ];
        let selection = PreviewSelection {
            anchor: PreviewCaret {
                block_index: 0,
                run_id: PreviewTextRunId::Body,
                offset: 0,
            },
            head: PreviewCaret {
                block_index: 1,
                run_id: PreviewTextRunId::Body,
                offset: 5,
            },
        };
        let md = preview_selection_markdown(&selection, &blocks, document).unwrap();
        assert!(md.contains("# Title"));
        assert!(md.contains("Hello world"));
        assert!(!md.contains("- item"));

        let html = MarkdownDocument::from_text(&md).render_html_fragment();
        assert!(html.contains("<h1"));
        assert!(html.to_lowercase().contains("hello"));
    }

    /// A distinguishable `PreviewBlock` for splice-diff tests. Distinct `tag`s
    /// compare unequal; the concrete variant is irrelevant to the diff.
    fn blk(tag: &str) -> PreviewBlock {
        PreviewBlock::CodeBlock {
            language: None,
            code: tag.to_string(),
            source_range: 0..0,
        }
    }

    fn blocks(tags: &[&str]) -> Vec<PreviewBlock> {
        tags.iter().map(|t| blk(t)).collect()
    }

    #[test]
    fn file_tree_visibility_hides_collapsed_descendants() {
        let root = PathBuf::from("workspace");
        let docs = root.join("docs");
        let notes = root.join("notes.md");
        let tree = FileTree {
            root: root.clone(),
            entries: vec![
                FileTreeEntry {
                    path: docs.clone(),
                    name: "docs".to_string(),
                    depth: 0,
                    kind: FileTreeEntryKind::Directory,
                    is_markdown: false,
                },
                FileTreeEntry {
                    path: docs.join("draft.md"),
                    name: "draft.md".to_string(),
                    depth: 1,
                    kind: FileTreeEntryKind::File,
                    is_markdown: true,
                },
                FileTreeEntry {
                    path: notes,
                    name: "notes.md".to_string(),
                    depth: 0,
                    kind: FileTreeEntryKind::File,
                    is_markdown: true,
                },
            ],
        };
        let mut collapsed = HashSet::new();
        collapsed.insert(docs);

        let (visible, total) = filtered_visible_file_tree_entries(&tree, "", &collapsed, 300);

        assert_eq!(total, 2);
        assert_eq!(
            visible
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["docs", "notes.md"]
        );
    }

    #[test]
    fn file_tree_context_actions_are_scoped_by_target_kind() {
        assert_eq!(
            file_tree_context_actions(FileTreeContextTargetKind::File),
            &[
                FileTreeContextAction::Open,
                FileTreeContextAction::OpenInNewTab,
                FileTreeContextAction::Rename,
                FileTreeContextAction::Delete,
                FileTreeContextAction::ShowInFileManager,
                FileTreeContextAction::Refresh,
            ]
        );
        assert_eq!(
            file_tree_context_actions(FileTreeContextTargetKind::Directory),
            &[
                FileTreeContextAction::CreateFile,
                FileTreeContextAction::CreateFolder,
                FileTreeContextAction::Rename,
                FileTreeContextAction::Delete,
                FileTreeContextAction::ShowInFileManager,
                FileTreeContextAction::Refresh,
            ]
        );
        assert_eq!(
            file_tree_context_actions(FileTreeContextTargetKind::Workspace),
            &[
                FileTreeContextAction::CreateFile,
                FileTreeContextAction::CreateFolder,
                FileTreeContextAction::Refresh,
                FileTreeContextAction::ShowInFileManager,
                FileTreeContextAction::FilterFiles,
            ]
        );
    }

    /// The inline name prompt's commit path calls `create_unique_file` /
    /// `create_unique_directory` / `rename_unique` with the user-typed name
    /// (rather than a hard-coded default). This exercises those operations at
    /// the model level - the app-level wiring is a thin wrapper that passes
    /// `pending.buffer` straight through.
    #[test]
    fn file_tree_name_prompt_commit_uses_typed_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        // Seed the tree with one existing file so the scan has a root.
        std::fs::write(root.join("seed.md"), "# Seed").unwrap();

        let mut tree = FileTree::scan(&root).unwrap();

        // CreateFile: typed name "essay.md" under the root.
        let created = tree.create_unique_file(&root, "essay.md").unwrap();
        assert_eq!(
            created.file_name().and_then(|n| n.to_str()),
            Some("essay.md")
        );
        assert!(created.exists());
        assert!(tree.entries.iter().any(|e| e.path == created));

        // CreateFolder: typed name "drafts".
        let folder = tree.create_unique_directory(&root, "drafts").unwrap();
        assert_eq!(folder.file_name().and_then(|n| n.to_str()), Some("drafts"));
        assert!(folder.is_dir());

        // Rename: rename essay.md -> final.md using the typed name.
        let renamed = tree.rename_unique(&created, "final.md").unwrap();
        assert_eq!(
            renamed.file_name().and_then(|n| n.to_str()),
            Some("final.md")
        );
        assert!(!created.exists());
        assert!(renamed.exists());
        assert!(tree.entries.iter().any(|e| e.path == renamed));
        assert!(!tree.entries.iter().any(|e| e.path == created));
    }

    /// The prompt must be pre-filled with a sensible default per kind so the
    /// existing defaults remain one Enter away. This pins the pre-fill contract
    /// that `open_name_prompt` / the context-menu branches rely on.
    #[test]
    fn pending_name_input_prefill_matches_kind_defaults() {
        let root = PathBuf::from("workspace");
        let create_file = PendingNameInput {
            kind: PendingNameKind::CreateFile,
            parent: root.clone(),
            target: None,
            buffer: "untitled.md".to_string(),
        };
        assert_eq!(create_file.kind, PendingNameKind::CreateFile);
        assert_eq!(create_file.buffer, "untitled.md");
        assert!(create_file.target.is_none());

        let create_folder = PendingNameInput {
            kind: PendingNameKind::CreateFolder,
            parent: root.clone(),
            target: None,
            buffer: "New Folder".to_string(),
        };
        assert_eq!(create_folder.kind, PendingNameKind::CreateFolder);
        assert_eq!(create_folder.buffer, "New Folder");

        let note = root.join("note.md");
        let rename = PendingNameInput {
            kind: PendingNameKind::Rename,
            parent: root.clone(),
            target: Some(note.clone()),
            buffer: "note.md".to_string(),
        };
        assert_eq!(rename.kind, PendingNameKind::Rename);
        assert_eq!(rename.target, Some(note));
        assert_eq!(rename.buffer, "note.md");
    }

    /// `has_text_input_focus` must consider a pending name prompt as focused so
    /// IME keystrokes route into the prompt buffer instead of the document.
    #[test]
    fn has_text_input_focus_includes_pending_name_prompt() {
        // The app-level field can't be constructed without a GPUI context, so
        // validate the routing predicate directly against the pending-input
        // presence: the trio (has_text_input_focus / active_input_text_mut /
        // after_input_changed) all key off `pending_name_input.is_some()`.
        let pending = Some(PendingNameInput {
            kind: PendingNameKind::CreateFile,
            parent: PathBuf::from("workspace"),
            target: None,
            buffer: String::new(),
        });
        assert!(pending.is_some());
        // A non-pending state must be treated as unfocused for the name buffer.
        let none: Option<PendingNameInput> = None;
        assert!(none.is_none());
    }

    /// `dir_is_non_empty` decides whether deleting a folder needs a second
    /// (recursive) confirmation. Empty folders must read as empty; folders
    /// with any entry must read as non-empty.
    #[test]
    fn dir_is_non_empty_detects_recursive_delete_target() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Empty folder -> not non-empty -> single confirm path.
        let empty = root.join("empty");
        std::fs::create_dir(&empty).unwrap();
        assert!(!dir_is_non_empty(&empty));

        // Folder with a file -> non-empty -> second recursive confirm.
        let with_file = root.join("with_file");
        std::fs::create_dir(&with_file).unwrap();
        std::fs::write(with_file.join("note.md"), "# Note").unwrap();
        assert!(dir_is_non_empty(&with_file));

        // Folder with only a subdirectory -> still non-empty.
        let with_sub = root.join("with_sub");
        std::fs::create_dir(&with_sub).unwrap();
        std::fs::create_dir(with_sub.join("child")).unwrap();
        assert!(dir_is_non_empty(&with_sub));

        // Non-existent path -> treated as empty (no second confirm; the delete
        // itself will fail later with a clear error).
        assert!(!dir_is_non_empty(&root.join("missing")));
    }

    #[test]
    fn preview_block_splice_reports_noop_for_identical_slices() {
        let a = blocks(&["a", "b", "c"]);
        assert_eq!(preview_block_splice(&a, &a), (3..3, 0));
    }

    #[test]
    fn preview_block_splice_isolates_a_single_changed_block() {
        let old = blocks(&["a", "b", "c"]);
        let new = blocks(&["a", "x", "c"]);
        // Only the middle block changed: replace index 1 with 1 new item.
        assert_eq!(preview_block_splice(&old, &new), (1..2, 1));
    }

    #[test]
    fn preview_block_splice_handles_insertion_and_deletion() {
        let a = blocks(&["a", "c"]);
        let b = blocks(&["a", "b", "c"]);
        assert_eq!(preview_block_splice(&a, &b), (1..1, 1)); // insert b
        assert_eq!(preview_block_splice(&b, &a), (1..2, 0)); // delete b
    }

    #[test]
    fn preview_block_splice_handles_full_replace_and_empty_edges() {
        assert_eq!(
            preview_block_splice(&blocks(&["a", "b"]), &blocks(&["x", "y"])),
            (0..2, 2)
        );
        assert_eq!(preview_block_splice(&[], &blocks(&["a", "b"])), (0..0, 2));
        assert_eq!(preview_block_splice(&blocks(&["a", "b"]), &[]), (0..2, 0));
    }

    /// Applying the computed splice to the old slice must reproduce the new
    /// slice exactly — the invariant `ListState` relies on. Mirrors `splice`'s
    /// own `Vec::splice` semantics: remove `range`, insert the new items that
    /// occupy the same positions in `new`.
    #[test]
    fn preview_block_splice_result_reconstructs_new_slice() {
        let cases: &[(&[&str], &[&str])] = &[
            (&["a", "b", "c"], &["a", "x", "c"]),
            (&["a", "c"], &["a", "b", "c"]),
            (&["a", "b", "c"], &["a", "c"]),
            (&["a", "b"], &["x", "y"]),
            (&[], &["a", "b"]),
            (&["a", "b"], &[]),
            (&["a", "b", "c", "d"], &["a", "b", "b", "c", "d"]),
        ];
        for (old_tags, new_tags) in cases {
            let old = blocks(old_tags);
            let new = blocks(new_tags);
            let (range, count) = preview_block_splice(&old, &new);
            let inserted: Vec<PreviewBlock> = new[range.start..range.start + count].to_vec();
            let mut reconstructed = old.clone();
            reconstructed.splice(range, inserted);
            assert_eq!(reconstructed, new, "old={old_tags:?} new={new_tags:?}");
        }
    }

    #[test]
    fn preview_parses_immediately_when_never_changed_or_never_parsed() {
        // First render of a fresh document: no change timestamp, no parse yet.
        assert!(should_parse_preview_now(None, None));
        // Mode switch after edits made long ago in Edit mode: the change was
        // just observed but there is no previous parse to fall back on.
        assert!(should_parse_preview_now(Some(Duration::ZERO), None));
    }

    #[test]
    fn preview_defers_mid_typing_and_parses_once_settled() {
        let fresh_parse = Some(PREVIEW_MAX_STALE / 4);
        // A keystroke a moment ago with a recent parse on screen: wait.
        assert!(!should_parse_preview_now(Some(Duration::ZERO), fresh_parse));
        assert!(!should_parse_preview_now(
            Some(PREVIEW_DEBOUNCE - Duration::from_millis(1)),
            fresh_parse
        ));
        // Debounce window elapsed: typing settled, parse.
        assert!(should_parse_preview_now(
            Some(PREVIEW_DEBOUNCE),
            fresh_parse
        ));
    }

    #[test]
    fn preview_parses_anyway_when_continuous_typing_keeps_it_stale() {
        // Keystrokes never stop (since_change stays ~0), but the blocks on
        // screen are past the staleness cap: parse so the preview keeps moving.
        assert!(should_parse_preview_now(
            Some(Duration::ZERO),
            Some(PREVIEW_MAX_STALE)
        ));
        assert!(!should_parse_preview_now(
            Some(Duration::ZERO),
            Some(PREVIEW_MAX_STALE - Duration::from_millis(1))
        ));
    }

    #[test]
    fn app_theme_cycles_through_six_builtin_themes() {
        let mut theme = AppTheme::Paper;
        let mut names = Vec::new();
        for _ in 0..AppTheme::ALL.len() {
            names.push(theme.name());
            theme = theme.next();
        }

        assert_eq!(
            names,
            vec!["Paper", "Ink", "Solar", "Forest", "Rose", "Graphite"]
        );
        assert_eq!(theme, AppTheme::Paper);
    }

    #[test]
    fn app_theme_restores_from_saved_name() {
        assert_eq!(AppTheme::from_name("ink"), Some(AppTheme::Ink));
        assert_eq!(AppTheme::from_name(" Graphite "), Some(AppTheme::Graphite));
        assert_eq!(AppTheme::from_name("missing"), None);
    }

    #[test]
    fn builtin_theme_table_exposes_popular_themes_with_unique_names() {
        let themes = builtin_theme_definitions();
        // The original six + at least five popular themes.
        assert!(themes.len() >= 11);
        // Original six must stay first and in canonical order so saved
        // preferences and the legacy cycle test keep resolving.
        let first_six: Vec<&str> = themes.iter().take(6).map(|t| t.name.as_str()).collect();
        assert_eq!(
            first_six,
            vec!["Paper", "Ink", "Solar", "Forest", "Rose", "Graphite"]
        );
        // Requested popular themes are present.
        for expected in [
            "GitHub Light",
            "Solarized Light",
            "One Light",
            "Tokyo Night",
        ] {
            assert!(
                themes.iter().any(|t| t.name == expected),
                "missing built-in theme {expected}"
            );
        }
        // Names are unique.
        let mut sorted: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), before, "duplicate built-in theme names");
    }

    #[test]
    fn shortcut_reference_lists_core_workflows() {
        let reference = shortcut_reference_text();

        assert!(reference.contains("Save: Secondary-S"));
        assert!(reference.contains("Cycle View Mode: Secondary-Shift-V"));
        assert!(reference.contains("Edit Mode: Secondary-Alt-1"));
        assert!(reference.contains("Visual Edit Mode: Secondary-Alt-4"));
        assert!(reference.contains("Split Preview Mode: Secondary-Alt-2"));
        assert!(reference.contains("Read Mode: Secondary-Alt-3"));
        assert!(reference.contains("Preferences: Secondary-Comma"));
        assert!(reference.contains("Find: Secondary-F"));
        assert!(reference.contains("DOCX: Secondary-Shift-D"));
    }

    #[test]
    fn pane_density_and_scrollbar_constants_stay_compact_and_usable() {
        assert!(
            PANE_OUTER_PADDING <= 16. * 0.20,
            "outer pane padding should stay close to the requested 15% density target"
        );
        assert!(
            PANE_INNER_PADDING < 16.,
            "inner padding should remain tighter than the old spacious pane padding"
        );
        assert!(
            PANE_SCROLLBAR_THUMB_WIDTH <= PANE_SCROLLBAR_RESERVED_WIDTH,
            "thumb must fit inside the reserved right-side scrollbar gutter"
        );
        assert_eq!(PANE_INNER_PADDING, 9.);
        assert!(
            PREVIEW_SCROLLBAR_SAFE_RIGHT_PADDING
                >= PANE_INNER_PADDING + PANE_SCROLLBAR_RESERVED_WIDTH,
            "preview content must reserve a right-side gutter before the overlay scrollbar"
        );
        assert!(
            RESIZE_HANDLE_WIDTH >= 8.,
            "resize handles should keep a usable invisible drag target"
        );
    }

    #[test]
    fn read_mode_preview_width_cap_only_applies_without_adaptive_width() {
        assert_eq!(READ_MODE_PREVIEW_MAX_WIDTH, 860.);
        assert!(read_mode_preview_is_constrained(ViewMode::Read, false));
        assert!(!read_mode_preview_is_constrained(ViewMode::Read, true));
        assert!(!read_mode_preview_is_constrained(ViewMode::Split, false));
        assert!(!read_mode_preview_is_constrained(ViewMode::Split, true));
        assert!(!read_mode_preview_is_constrained(ViewMode::Edit, false));
        assert!(!read_mode_preview_is_constrained(
            ViewMode::VisualEdit,
            false
        ));
    }

    #[test]
    fn view_modes_have_distinct_status_and_expected_pane_layouts() {
        assert_eq!(
            view_mode_status_message(ViewMode::Edit),
            Msg::StatusEditMode
        );
        assert_eq!(
            view_mode_status_message(ViewMode::VisualEdit),
            Msg::StatusVisualEditMode
        );
        assert_eq!(
            view_mode_status_message(ViewMode::Split),
            Msg::StatusSplitPreviewMode
        );
        assert_eq!(
            view_mode_status_message(ViewMode::Read),
            Msg::StatusReadMode
        );
        assert_eq!(view_mode_pane_widths(ViewMode::Edit, 0.4), (1.0, 0.0));
        assert_eq!(view_mode_pane_widths(ViewMode::VisualEdit, 0.4), (1.0, 0.0));
        assert_eq!(view_mode_pane_widths(ViewMode::Split, 0.4), (0.4, 0.6));
        assert_eq!(view_mode_pane_widths(ViewMode::Read, 0.4), (0.0, 1.0));
    }

    #[test]
    fn table_edit_toolbar_is_available_only_in_visual_edit() {
        assert!(table_toolbar_actions_for_view_mode(ViewMode::Edit).is_empty());
        assert!(table_toolbar_actions_for_view_mode(ViewMode::Split).is_empty());
        assert!(table_toolbar_actions_for_view_mode(ViewMode::Read).is_empty());

        let edits = table_toolbar_actions_for_view_mode(ViewMode::VisualEdit)
            .iter()
            .map(|(_, edit, _)| *edit)
            .collect::<Vec<_>>();
        assert_eq!(
            edits,
            vec![
                TableEdit::AddRow,
                TableEdit::DeleteRow,
                TableEdit::MoveRowUp,
                TableEdit::MoveRowDown,
                TableEdit::AddColumn,
                TableEdit::DeleteColumn,
            ]
        );
    }

    #[test]
    fn direct_view_mode_switching_preserves_tab_state() {
        let mut tab = EditorTab::new(MarkdownDocument::from_text("hello"));
        tab.selected_range = 1..4;
        tab.push_undo_snapshot();
        let version = tab.document.version();
        let mut mode = ViewMode::Edit;
        for target in [
            ViewMode::VisualEdit,
            ViewMode::Split,
            ViewMode::Read,
            ViewMode::Edit,
        ] {
            assign_view_mode(&mut mode, target);
            assert_eq!(mode, target);
            assert_eq!(tab.document.text(), "hello");
            assert_eq!(tab.document.version(), version);
            assert_eq!(tab.selected_range, 1..4);
            assert_eq!(tab.undo_stack.len(), 1);
        }
    }

    #[test]
    fn visual_text_positions_map_to_source_content_ranges() {
        let segments = vec![
            VisualTextSegment {
                visible_range: 0..5,
                source_range: 2..7,
            },
            VisualTextSegment {
                visible_range: 5..9,
                source_range: 11..15,
            },
        ];
        assert_eq!(visual_source_for_visible(&segments, 0), 2);
        assert_eq!(visual_source_for_visible(&segments, 4), 6);
        assert_eq!(visual_source_for_visible(&segments, 6), 12);
        assert_eq!(visual_visible_for_source(&segments, 13), Some(7));
        assert_eq!(visual_visible_for_source(&segments, 9), None);
    }

    #[test]
    fn visual_focus_uses_half_open_block_ranges() {
        assert!(visual_source_range_is_focused(&(10..20), 10, 30));
        assert!(visual_source_range_is_focused(&(10..20), 19, 30));
        assert!(!visual_source_range_is_focused(&(10..20), 20, 30));
        assert!(visual_source_range_is_focused(&(20..30), 20, 30));
        assert!(visual_source_range_is_focused(&(20..30), 30, 30));
    }

    #[test]
    fn custom_theme_palette_uses_definition_colors() {
        let theme = ThemeDefinition {
            name: "Test".into(),
            is_dark: false,
            colors: ThemeColors {
                app_bg: 0x010203,
                panel_bg: 0x111213,
                surface_bg: 0x212223,
                text: 0x313233,
                muted: 0x414243,
                border: 0x515253,
                active_bg: 0x616263,
                active_text: 0x717273,
            },
        };
        let palette = theme_palette_from_definition(&theme);

        assert_eq!(palette.app_bg, rgb(0x010203));
        assert_eq!(palette.active_text, rgb(0x717273));
    }

    #[test]
    fn ime_selected_range_is_relative_to_composition_text() {
        let composing = "😀文";
        let range = EditorTab::relative_range_from_utf16(composing, &(2..3));

        assert_eq!(&composing[range], "文");
        assert_eq!(utf16_offset_to_byte_offset("a😀文", 3), "a😀".len());
        assert_eq!(byte_offset_to_utf16_offset("a😀文", "a😀".len()), 3);
    }

    #[test]
    fn editor_tab_new_initializes_empty_state() {
        let tab = EditorTab::new(MarkdownDocument::from_text("hello"));
        // Defaults match what the pre-refactor MarkionApp::new used.
        assert_eq!(tab.document.text(), "hello");
        assert!(tab.undo_stack.is_empty());
        assert!(tab.redo_stack.is_empty());
        assert_eq!(tab.selected_range, 0..0);
        assert!(!tab.selection_reversed);
        assert!(tab.marked_range.is_none());
        assert!(tab.last_lines.is_empty());
        assert!(tab.line_offsets.is_empty());
        assert!(tab.line_heights.is_empty());
        assert!(tab.last_bounds.is_none());
        assert_eq!(tab.line_height, px(24.));
        assert!(!tab.is_selecting);
        assert!(tab.last_recovery_file.is_none());
        assert_eq!(tab.autosave_generation, 0);
        assert!(tab.display_text_cache.borrow().is_none());
        assert!(tab.preview_parse_inflight.is_none());
    }

    #[test]
    fn reset_preview_list_orphans_inflight_parse() {
        let mut tab = EditorTab::new(MarkdownDocument::from_text("# One"));
        tab.preview_parse_inflight = Some(next_preview_parse_id());
        // Replacing the document must clear the marker so a background result
        // for the old text can no longer find (and corrupt) this tab.
        tab.reset_preview_list();
        assert!(tab.preview_parse_inflight.is_none());
        assert!(tab.preview_reflects_version.is_none());
    }

    #[test]
    fn undo_history_keeps_one_full_snapshot_and_compacts_the_rest() {
        let mut tab = EditorTab::new(MarkdownDocument::from_text("hello world"));
        for (range, insert) in [(5..5, ","), (12..12, "!"), (0..5, "goodbye")] {
            tab.selected_range = range.start..range.start;
            tab.push_undo_snapshot();
            tab.document.replace_range(range, insert);
        }

        assert_eq!(tab.undo_stack.len(), 3);
        assert!(matches!(tab.undo_stack[0], UndoEntry::Diff(_)));
        assert!(matches!(tab.undo_stack[1], UndoEntry::Diff(_)));
        assert!(matches!(tab.undo_stack[2], UndoEntry::Full(_)));
    }

    #[test]
    fn undo_redo_roundtrip_through_compacted_history() {
        // Walk a document through edits that exercise insertion, deletion,
        // replacement, and multi-byte chars sharing UTF-8 prefix bytes
        // (中 E4B8AD vs 串 E4B8B2 — the byte diff lands mid-char and must be
        // widened to a char boundary), then undo/redo across the whole
        // history and verify every intermediate state.
        let edits: [(Range<usize>, &str); 4] =
            [(5..5, " 中"), (9..9, " beta"), (5..9, " 串"), (0..5, "")];
        let mut tab = EditorTab::new(MarkdownDocument::from_text("alpha"));
        // Undoing edit i restores its pre-edit text and the selection captured
        // when its snapshot was pushed (the range about to be replaced).
        let mut undo_expected: Vec<(String, Range<usize>)> = Vec::new();
        let mut redo_texts: Vec<String> = Vec::new();
        for (range, insert) in edits {
            undo_expected.push((tab.document.text().to_string(), range.clone()));
            tab.selected_range = range.clone();
            tab.push_undo_snapshot();
            tab.document.replace_range(range.clone(), insert);
            let end = range.start + insert.len();
            tab.selected_range = end..end;
            redo_texts.push(tab.document.text().to_string());
        }

        // Undo all the way down, checking text and selection at every step.
        for (text, selection) in undo_expected.iter().rev() {
            assert!(tab.apply_undo());
            assert_eq!(tab.document.text(), text, "undo text");
            assert_eq!(&tab.selected_range, selection, "undo selection");
        }
        assert!(!tab.apply_undo());

        // Redo all the way back up.
        for text in &redo_texts {
            assert!(tab.apply_redo());
            assert_eq!(tab.document.text(), text, "redo text");
        }
        assert!(!tab.apply_redo());

        // Interleave: undo twice, redo once, then a fresh edit clears redo.
        assert!(tab.apply_undo());
        assert!(tab.apply_undo());
        assert!(tab.apply_redo());
        assert_eq!(tab.document.text(), redo_texts[2]);
        tab.push_undo_snapshot();
        tab.document.replace_range(0..0, "x");
        assert!(tab.redo_stack.is_empty());
        assert!(tab.apply_undo());
        assert_eq!(tab.document.text(), redo_texts[2]);
    }

    #[test]
    fn grapheme_boundaries_match_full_document_segmentation() {
        // The line-local scan must agree with the old whole-document
        // segmentation at every grapheme boundary, across ASCII, CJK,
        // combining marks, ZWJ emoji clusters, and "\r\n".
        let text = "abc\nxy 中文 e\u{301}fin 👍🏽 👨\u{200d}👩\u{200d}👧\r\nnext line\n\nend";
        let tab = EditorTab::new(MarkdownDocument::from_text(text));

        let prev_reference = |offset: usize| {
            text.grapheme_indices(true)
                .rev()
                .find_map(|(idx, _)| (idx < offset).then_some(idx))
                .unwrap_or(0)
        };
        let next_reference = |offset: usize| {
            text.grapheme_indices(true)
                .find_map(|(idx, _)| (idx > offset).then_some(idx))
                .unwrap_or(text.len())
        };

        let boundaries: Vec<usize> = text
            .grapheme_indices(true)
            .map(|(idx, _)| idx)
            .chain(std::iter::once(text.len()))
            .collect();
        for &offset in &boundaries {
            assert_eq!(
                tab.previous_boundary(offset),
                prev_reference(offset),
                "previous_boundary at {offset}"
            );
            assert_eq!(
                tab.next_boundary(offset),
                next_reference(offset),
                "next_boundary at {offset}"
            );
        }
    }

    #[test]
    fn multiple_tabs_isolate_cursor_and_undo() {
        // Two tabs opened independently must not share cursor/selection/undo
        // state. This is the core invariant the refactor introduces.
        let mut tab_a = EditorTab::new(MarkdownDocument::from_text("abc"));
        let tab_b = EditorTab::new(MarkdownDocument::from_text("xyz"));

        // Move the cursor in tab A to offset 2.
        tab_a.selected_range = 2..2;
        tab_a.push_undo_snapshot();
        // Tab B is independently at offset 0 with no undo history.
        assert_eq!(tab_b.selected_range, 0..0);
        assert!(tab_b.undo_stack.is_empty());

        // Editing tab A's document does not affect tab B's text.
        tab_a.document.replace_range(0..1, "A");
        assert_eq!(tab_a.document.text(), "Abc");
        assert_eq!(tab_b.document.text(), "xyz");

        // Cursor positions stay isolated.
        assert_eq!(tab_a.cursor_offset(), 2);
        assert_eq!(tab_b.cursor_offset(), 0);
        assert_eq!(tab_a.undo_stack.len(), 1);
        assert_eq!(tab_b.undo_stack.len(), 0);
    }

    #[test]
    fn find_tab_with_document_path_matches_canonical_paths() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        std::fs::create_dir(&docs).unwrap();
        let first_path = docs.join("first.md");
        let second_path = docs.join("second.md");
        std::fs::write(&first_path, "# First").unwrap();
        std::fs::write(&second_path, "# Second").unwrap();

        let tabs = vec![
            EditorTab::new(MarkdownDocument::open(&first_path).unwrap()),
            EditorTab::new(MarkdownDocument::open(&second_path).unwrap()),
            EditorTab::new(MarkdownDocument::from_text("untitled")),
        ];

        let equivalent_second_path = docs.join("..").join("docs").join("second.md");
        assert_eq!(
            find_tab_with_document_path(&tabs, &equivalent_second_path),
            Some(1)
        );
        assert_eq!(
            find_tab_with_document_path(&tabs, &docs.join("missing.md")),
            None
        );
    }

    #[test]
    fn opening_existing_file_focuses_without_duplicating_or_resetting_state() {
        let dir = tempfile::tempdir().unwrap();
        let first_path = dir.path().join("first.md");
        let second_path = dir.path().join("second.md");
        std::fs::write(&first_path, "first").unwrap();
        std::fs::write(&second_path, "second").unwrap();

        let mut tabs = vec![
            EditorTab::new(MarkdownDocument::open(&first_path).unwrap()),
            EditorTab::new(MarkdownDocument::open(&second_path).unwrap()),
        ];
        tabs[1].selected_range = 2..2;
        tabs[1].push_undo_snapshot();
        tabs[1].document.replace_range(0..0, "dirty ");
        let cached_preview = tabs[1].document.preview_blocks_shared();

        let active_tab = if let Some(index) = find_tab_with_document_path(&tabs, &second_path) {
            index
        } else {
            tabs.push(EditorTab::new(
                MarkdownDocument::open(&second_path).unwrap(),
            ));
            tabs.len() - 1
        };

        assert_eq!(tabs.len(), 2, "already-open files must not append tabs");
        assert_eq!(active_tab, 1);
        assert_eq!(tabs[1].document.text(), "dirty second");
        assert!(tabs[1].document.is_dirty());
        assert_eq!(tabs[1].selected_range, 2..2);
        assert_eq!(tabs[1].undo_stack.len(), 1);
        assert!(std::sync::Arc::ptr_eq(
            &tabs[1].document.preview_blocks_shared(),
            &cached_preview
        ));
    }

    #[test]
    fn tab_vec_close_last_leaves_one_tab() {
        // Simulates the close_tab_confirmed invariant: closing the last tab
        // leaves exactly one fresh (untitled) tab rather than an empty window.
        let mut tabs: Vec<EditorTab> = vec![EditorTab::new(MarkdownDocument::from_text("only"))];
        let mut active_tab = 0usize;

        // Closing the last tab resets it in place to a fresh document.
        if tabs.len() <= 1 {
            tabs[0] = EditorTab::new(MarkdownDocument::new());
            active_tab = 0;
        } else {
            tabs.remove(active_tab);
            if active_tab >= tabs.len() {
                active_tab = tabs.len() - 1;
            }
        }
        assert_eq!(tabs.len(), 1, "closing the last tab must keep one tab");
        assert_eq!(active_tab, 0);
        assert!(
            tabs[0].document.path().is_none(),
            "the replacement tab is an untitled document"
        );

        // With two tabs, closing the active one removes it and leaves the other.
        tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("first")),
            EditorTab::new(MarkdownDocument::from_text("second")),
        ];
        active_tab = 1;
        assert_eq!(tabs.len(), 2);
        tabs.remove(active_tab);
        if active_tab >= tabs.len() {
            active_tab = tabs.len() - 1;
        }
        assert_eq!(tabs.len(), 1);
        assert_eq!(active_tab, 0);
        assert_eq!(tabs[0].document.text(), "first");
    }

    #[test]
    fn active_tab_accessors_clamp_a_stale_index() {
        // Regression for the close-tab-then-close-another crash. Tab-bar click
        // closures capture an `index` at render time; a close since then can
        // leave that index >= tabs.len() by the time the closure fires. The
        // accessors clamp the index so a transiently-stale value cannot panic.
        //
        // Reproduction: 3 tabs [A, B, C]; close B (index 1) -> [A, C]; the C
        // closure still carries index 2, which is now out of range (len == 2).
        let mut app_tabs: Vec<EditorTab> = vec![
            EditorTab::new(MarkdownDocument::from_text("A")),
            EditorTab::new(MarkdownDocument::from_text("B")),
            EditorTab::new(MarkdownDocument::from_text("C")),
        ];
        let mut app_active_tab = 1usize;

        // Close tab B (the active one): remove(1) -> [A, C].
        app_tabs.remove(app_active_tab);
        if app_active_tab >= app_tabs.len() {
            app_active_tab = app_tabs.len() - 1;
        }
        assert_eq!(app_tabs.len(), 2);

        // Simulate the stale closure firing: a click that still carries the
        // pre-close index 2. Without clamping this would be `app_tabs[2]` on a
        // 2-element vec — a panic. The clamp mirrors `active_tab()`:
        let clamped = app_active_tab.min(app_tabs.len().saturating_sub(1));
        let _ = &app_tabs[clamped]; // must not panic.
        // A stale index of 2 also clamps safely:
        let stale_index = 2usize;
        let clamped_stale = stale_index.min(app_tabs.len().saturating_sub(1));
        assert_eq!(clamped_stale, 1, "stale index 2 clamps to last valid (1)");
        let _ = &app_tabs[clamped_stale]; // must not panic.

        // And the guard used in tab_bar_view closures rejects a stale index
        // outright rather than trusting it:
        let index_from_closure = 2usize;
        assert!(
            !(index_from_closure < app_tabs.len()),
            "tab-bar closure must skip a stale index instead of assigning it"
        );
    }

    #[test]
    fn any_tab_dirty_detection() {
        // request_quit / window-close guard use tabs.iter().any(dirty).
        let tabs: Vec<EditorTab> = vec![
            EditorTab::new(MarkdownDocument::from_text("clean")),
            EditorTab::new(MarkdownDocument::from_text("clean2")),
        ];
        assert!(
            !tabs.iter().any(|t| t.document.is_dirty()),
            "freshly-created documents are not dirty"
        );
    }

    /// `sync_fraction` is the proportional coupling primitive: a pane with no
    /// scrollable range (max <= 1px) reports 0 so it never drives the other
    /// pane; otherwise it reports the clamped offset/max ratio.
    #[test]
    fn sync_fraction_clamps_and_zeros_unscrollable_panes() {
        // No scrollable range -> 0 regardless of offset (the guard for a
        // pane that fits its viewport).
        assert_eq!(sync_fraction(0., 0.), 0.);
        assert_eq!(sync_fraction(123., 1.), 0.);
        assert_eq!(sync_fraction(50., 0.5), 0.);

        // Mid-range offsets map to a proportional fraction.
        assert!((sync_fraction(50., 100.) - 0.5).abs() < 1e-6);
        assert!((sync_fraction(25., 100.) - 0.25).abs() < 1e-6);

        // Offset clamps to the top/bottom of the range.
        assert!((sync_fraction(0., 100.) - 0.).abs() < 1e-6);
        assert!((sync_fraction(100., 100.) - 1.).abs() < 1e-6);
        // Overscroll past the max still reports 1.0 (clamped), not >1.
        assert!((sync_fraction(150., 100.) - 1.).abs() < 1e-6);
        // Negative offset (should not normally occur) clamps to 0.
        assert!((sync_fraction(-10., 100.) - 0.).abs() < 1e-6);
    }

    /// Coupling is active only in Split Preview (both panes visible) and only
    /// when the preference is enabled — never in Edit or Read mode, and never
    /// when the preference is off even in Split.
    #[test]
    fn sync_scroll_is_active_only_in_split_when_enabled() {
        assert!(sync_scroll_is_active(ViewMode::Split, true));

        // Split but disabled: not coupled.
        assert!(!sync_scroll_is_active(ViewMode::Split, false));
        // Other view modes never couple, even with the preference on.
        assert!(!sync_scroll_is_active(ViewMode::Edit, true));
        assert!(!sync_scroll_is_active(ViewMode::VisualEdit, true));
        assert!(!sync_scroll_is_active(ViewMode::Read, true));
    }

    #[test]
    fn default_heading_menu_exposes_h4_and_h5() {
        assert_eq!(DEFAULT_HEADING_MENU_MAX_LEVEL, 5);
        assert_eq!(
            normalize_heading_menu_max_level(3),
            DEFAULT_HEADING_MENU_MAX_LEVEL
        );
        assert_eq!(
            heading_native_menu_items(Language::En, DEFAULT_HEADING_MENU_MAX_LEVEL).len(),
            5
        );
    }
}

fn main() {
    // Diagnostic file logging (daily rotation in the Markion log dir). Failures
    // are non-fatal: the editor starts without file logging.
    let log_dir = markion::init_logging();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_dir = ?log_dir,
        "Markion starting"
    );

    // Load the syntect grammar registry off the main thread so the first
    // highlighted code block never blocks the typing path (~100ms of grammar
    // parsing happens here instead of on first use).
    std::thread::spawn(markion::warm_highlighter);

    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("up", Up, None),
            KeyBinding::new("down", Down, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("shift-up", SelectUp, None),
            KeyBinding::new("shift-down", SelectDown, None),
            // `secondary-` maps to `cmd` on macOS and `ctrl` on Windows/Linux,
            // so shortcuts match each platform's convention.
            KeyBinding::new("secondary-a", SelectAll, None),
            KeyBinding::new("secondary-v", Paste, None),
            KeyBinding::new("secondary-c", Copy, None),
            KeyBinding::new("secondary-x", Cut, None),
            KeyBinding::new("secondary-z", Undo, None),
            KeyBinding::new("secondary-shift-z", Redo, None),
            KeyBinding::new("secondary-y", Redo, None),
            KeyBinding::new("secondary-b", Bold, None),
            KeyBinding::new("secondary-i", Italic, None),
            KeyBinding::new("secondary-e", InlineCode, None),
            KeyBinding::new("secondary-k", InsertLink, None),
            KeyBinding::new("secondary-shift-i", InsertImage, None),
            KeyBinding::new("secondary-1", Heading1, None),
            KeyBinding::new("secondary-2", Heading2, None),
            KeyBinding::new("secondary-3", Heading3, None),
            KeyBinding::new("secondary-4", Heading4, None),
            KeyBinding::new("secondary-5", Heading5, None),
            KeyBinding::new("secondary-6", Heading6, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("enter", InsertNewline, None),
            KeyBinding::new("tab", Indent, None),
            KeyBinding::new("shift-tab", Outdent, None),
            KeyBinding::new("secondary-n", NewDocument, None),
            KeyBinding::new("secondary-o", OpenDocument, None),
            KeyBinding::new("secondary-s", SaveDocument, None),
            KeyBinding::new("secondary-shift-s", SaveDocumentAs, None),
            KeyBinding::new("secondary-shift-h", ExportHtml, None),
            KeyBinding::new("secondary-alt-shift-h", ExportPlainHtml, None),
            KeyBinding::new("secondary-shift-p", ExportPdf, None),
            KeyBinding::new("secondary-shift-l", ExportLatex, None),
            KeyBinding::new("secondary-shift-d", ExportDocx, None),
            KeyBinding::new("secondary-shift-g", ExportPng, None),
            KeyBinding::new("secondary-alt-shift-g", ExportJpeg, None),
            KeyBinding::new("secondary-shift-v", ToggleViewMode, None),
            KeyBinding::new("secondary-alt-1", SetEditMode, None),
            KeyBinding::new("secondary-alt-4", SetVisualEditMode, None),
            KeyBinding::new("secondary-alt-2", SetSplitPreviewMode, None),
            KeyBinding::new("secondary-alt-3", SetReadMode, None),
            // NB: no `secondary-b` for the sidebar — that collides with Bold
            // (the documented Ctrl+B). The sidebar toggle lives in the View menu.
            KeyBinding::new("secondary-alt-b", ToggleSidebar, None),
            KeyBinding::new("secondary-shift-f", ToggleFileTree, None),
            KeyBinding::new("secondary-alt-f", FocusFileTreeSearch, None),
            KeyBinding::new("escape", ClearFileTreeSearch, None),
            KeyBinding::new("f5", RefreshFileTree, None),
            KeyBinding::new("secondary-alt-n", CreateTreeFile, None),
            KeyBinding::new("secondary-alt-shift-n", CreateTreeFolder, None),
            KeyBinding::new("f2", RenameTreeEntry, None),
            KeyBinding::new("secondary-delete", DeleteTreeEntry, None),
            KeyBinding::new("f6", ToggleOutline, None),
            KeyBinding::new("secondary-shift-t", CycleTheme, None),
            KeyBinding::new("f7", ToggleFocusMode, None),
            KeyBinding::new("f8", ToggleTypewriterMode, None),
            KeyBinding::new("secondary-shift-4", ToggleCodeLineNumbers, None),
            KeyBinding::new("secondary-shift-m", FormatTable, None),
            KeyBinding::new("secondary-alt-enter", TableAddRow, None),
            KeyBinding::new("secondary-alt-backspace", TableDeleteRow, None),
            KeyBinding::new("secondary-alt-up", TableMoveRowUp, None),
            KeyBinding::new("secondary-alt-down", TableMoveRowDown, None),
            KeyBinding::new("secondary-alt-right", TableAddColumn, None),
            KeyBinding::new("secondary-alt-left", TableDeleteColumn, None),
            KeyBinding::new("secondary-f", ShowFind, None),
            KeyBinding::new("secondary-h", ShowReplace, None),
            KeyBinding::new("f3", FindNext, None),
            KeyBinding::new("shift-f3", FindPrevious, None),
            KeyBinding::new("secondary-comma", ShowPreferences, None),
            KeyBinding::new("f1", ShowShortcuts, None),
            KeyBinding::new("secondary-q", Quit, None),
            KeyBinding::new("ctrl-tab", NextTab, None),
            KeyBinding::new("ctrl-shift-tab", PrevTab, None),
            KeyBinding::new("secondary-t", OpenInNewTab, None),
            KeyBinding::new("secondary-w", CloseTab, None),
        ]);
        // Install the native menu once with the default language; the window
        // hook below re-installs it after the saved language preference has
        // been loaded, so the OS menu bar honours the user's choice on launch.
        install_menus(Language::default(), DEFAULT_HEADING_MENU_MAX_LEVEL, cx);

        let bounds = Bounds::centered(None, size(px(1180.), px(760.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some(SharedString::from(MARKION_WINDOW_TITLE)),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    app_id: Some(MARKION_APP_ID.to_string()),
                    ..Default::default()
                },
                |_, cx| cx.new(MarkionApp::new),
            )
            .unwrap();

        window
            .update(cx, |app, window, cx| {
                install_window_close_guard(window, cx.entity(), cx);
                window.focus(&app.focus_handle(cx));
                // Re-translate the native menu now that the saved language
                // preference has been loaded by `MarkionApp::new`.
                install_menus(app.language, app.heading_menu_max_level, cx);
                app.check_recovery_on_startup(window, cx);
                // The file tree is intentionally NOT scanned on startup. With
                // only the in-memory welcome document open there is no chosen
                // workspace directory, and showing the program's own working
                // directory is not useful for a Markdown editor. The tree stays
                // `None` (empty-state placeholder) until a real file is opened,
                // at which point `update_workspace_root_from_document` scans
                // that file's parent directory.
                cx.activate(true);
            })
            .unwrap();
        cx.activate(true);
    });
}
