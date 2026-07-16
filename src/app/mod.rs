use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    env,
    ffi::OsString,
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::prelude::*;
use gpui::{
    App, Application, Bounds, ClickEvent, ClipboardItem, Context, CursorStyle, DefiniteLength,
    DispatchPhase, Div, DragMoveEvent, Element, ElementId, ElementInputHandler, Empty, Entity,
    EntityInputHandler, ExternalPaths, FocusHandle, Focusable, FontStyle, FontWeight,
    GlobalElementId, HighlightStyle, Hitbox, HitboxBehavior, Image, ImageFormat, ImageSource,
    KeyBinding, LayoutId, ListAlignment, ListState, Menu, MenuItem, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PaintQuad, PathPromptOptions, Pixels, Point, PromptButton,
    PromptLevel, Rgba, ScrollHandle, SharedString, Stateful, StrikethroughStyle, Style, StyledText,
    TextLayout, TextRun, Timer, TitlebarOptions, UTF16Selection, UnderlineStyle, Window,
    WindowBounds, WindowOptions, WrappedLine, actions, anchored, canvas, div, fill, img, list,
    point, px, rgb, rgba, size,
};
use markion::{
    AppPreferences, AutoSavePreferences, AutosaveOutcome, DEFAULT_HEADING_MENU_MAX_LEVEL,
    EXTENDED_HEADING_MENU_MAX_LEVEL, ExportBackend, ExportFormat, ExportPreferences, FileTree,
    FileTreeEntry, FileTreeEntryKind, HighlightKind, HighlightedSpan, HtmlPreviewPart, Language,
    MarkdownDocument, MarkdownFormat, Msg, PreviewBlock, RichText, SearchMatchRange, SearchOptions,
    ShortcutCategory, ShortcutPlatform, SidebarTab, TableEdit, ThemeColors, ThemeDefinition,
    ViewMode, VisualBlock, VisualBlockKind, VisualInlineRun, VisualSourceIslandKind,
    builtin_diagram_registry, builtin_theme_definitions, default_preferences_path,
    default_recovery_dir, default_themes_dir, delete_recovery_file, diagram_backend_id,
    highlight_code, html_preview_parts, html_preview_plain_text, is_markdown_path,
    list_recovery_files, list_theme_definitions, load_app_preferences, load_recovery_file,
    normalize_heading_menu_max_level, render_math, save_app_preferences, save_theme_definition,
    shortcut_catalog, sidebar_tab_label, t, tf, title_from_path,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppMenu {
    File,
    Edit,
    View,
    Format,
    Export,
    Help,
}

fn menu_after_hover(active: Option<AppMenu>, hovered: AppMenu) -> Option<AppMenu> {
    active.map(|_| hovered)
}

/// One source of truth for a keyboard binding and the text shown beside its
/// in-window menu item. Explicit platform labels keep GPUI's internal
/// `secondary` modifier out of user-facing chrome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MenuShortcut {
    binding: &'static str,
    windows_linux: &'static str,
    macos: &'static str,
}

impl MenuShortcut {
    const fn new(binding: &'static str, windows_linux: &'static str, macos: &'static str) -> Self {
        Self {
            binding,
            windows_linux,
            macos,
        }
    }

    const fn label(self, platform: ShortcutPlatform) -> &'static str {
        match platform {
            ShortcutPlatform::WindowsLinux => self.windows_linux,
            ShortcutPlatform::MacOS => self.macos,
        }
    }
}

/// Shared descriptors for actions that appear in the six application menus.
/// Unbound menu actions intentionally have no entry in this module.
mod menu_shortcuts {
    use super::MenuShortcut;

    pub const NEW_DOCUMENT: MenuShortcut = MenuShortcut::new("secondary-n", "Ctrl+N", "Cmd+N");
    pub const OPEN_DOCUMENT: MenuShortcut = MenuShortcut::new("secondary-o", "Ctrl+O", "Cmd+O");
    pub const SAVE_DOCUMENT: MenuShortcut = MenuShortcut::new("secondary-s", "Ctrl+S", "Cmd+S");
    pub const SAVE_DOCUMENT_AS: MenuShortcut =
        MenuShortcut::new("secondary-shift-s", "Ctrl+Shift+S", "Cmd+Shift+S");
    pub const OPEN_IN_NEW_TAB: MenuShortcut = MenuShortcut::new("secondary-t", "Ctrl+T", "Cmd+T");
    pub const CLOSE_TAB: MenuShortcut = MenuShortcut::new("secondary-w", "Ctrl+W", "Cmd+W");
    pub const NEXT_TAB: MenuShortcut = MenuShortcut::new("ctrl-tab", "Ctrl+Tab", "Ctrl+Tab");
    pub const PREV_TAB: MenuShortcut =
        MenuShortcut::new("ctrl-shift-tab", "Ctrl+Shift+Tab", "Ctrl+Shift+Tab");
    pub const SHOW_PREFERENCES: MenuShortcut =
        MenuShortcut::new("secondary-comma", "Ctrl+,", "Cmd+,");
    pub const QUIT: MenuShortcut = MenuShortcut::new("secondary-q", "Ctrl+Q", "Cmd+Q");

    pub const UNDO: MenuShortcut = MenuShortcut::new("secondary-z", "Ctrl+Z", "Cmd+Z");
    pub const REDO: MenuShortcut = MenuShortcut::new("secondary-y", "Ctrl+Y", "Cmd+Y");
    pub const COPY: MenuShortcut = MenuShortcut::new("secondary-c", "Ctrl+C", "Cmd+C");
    pub const CUT: MenuShortcut = MenuShortcut::new("secondary-x", "Ctrl+X", "Cmd+X");
    pub const PASTE: MenuShortcut = MenuShortcut::new("secondary-v", "Ctrl+V", "Cmd+V");
    pub const SELECT_ALL: MenuShortcut = MenuShortcut::new("secondary-a", "Ctrl+A", "Cmd+A");

    pub const TOGGLE_VIEW_MODE: MenuShortcut =
        MenuShortcut::new("secondary-shift-v", "Ctrl+Shift+V", "Cmd+Shift+V");
    pub const SET_EDIT_MODE: MenuShortcut =
        MenuShortcut::new("secondary-alt-1", "Ctrl+Alt+1", "Cmd+Option+1");
    pub const SET_VISUAL_EDIT_MODE: MenuShortcut =
        MenuShortcut::new("secondary-alt-4", "Ctrl+Alt+4", "Cmd+Option+4");
    pub const SET_SPLIT_PREVIEW_MODE: MenuShortcut =
        MenuShortcut::new("secondary-alt-2", "Ctrl+Alt+2", "Cmd+Option+2");
    pub const SET_READ_MODE: MenuShortcut =
        MenuShortcut::new("secondary-alt-3", "Ctrl+Alt+3", "Cmd+Option+3");
    pub const TOGGLE_SIDEBAR: MenuShortcut =
        MenuShortcut::new("secondary-shift-b", "Ctrl+Shift+B", "Cmd+Shift+B");
    pub const TOGGLE_FILE_TREE: MenuShortcut =
        MenuShortcut::new("secondary-shift-f", "Ctrl+Shift+F", "Cmd+Shift+F");
    pub const TOGGLE_OUTLINE: MenuShortcut = MenuShortcut::new("f6", "F6", "F6");
    pub const TOGGLE_FOCUS_MODE: MenuShortcut = MenuShortcut::new("f7", "F7", "F7");
    pub const TOGGLE_TYPEWRITER_MODE: MenuShortcut = MenuShortcut::new("f8", "F8", "F8");
    pub const TOGGLE_CODE_LINE_NUMBERS: MenuShortcut =
        MenuShortcut::new("secondary-shift-4", "Ctrl+Shift+4", "Cmd+Shift+4");
    pub const SHOW_FIND: MenuShortcut = MenuShortcut::new("secondary-f", "Ctrl+F", "Cmd+F");
    pub const SHOW_REPLACE: MenuShortcut = MenuShortcut::new("secondary-h", "Ctrl+H", "Cmd+H");
    pub const FIND_NEXT: MenuShortcut = MenuShortcut::new("f3", "F3", "F3");
    pub const FIND_PREVIOUS: MenuShortcut = MenuShortcut::new("shift-f3", "Shift+F3", "Shift+F3");
    pub const CYCLE_THEME: MenuShortcut =
        MenuShortcut::new("secondary-shift-t", "Ctrl+Shift+T", "Cmd+Shift+T");

    pub const BOLD: MenuShortcut = MenuShortcut::new("secondary-b", "Ctrl+B", "Cmd+B");
    pub const ITALIC: MenuShortcut = MenuShortcut::new("secondary-i", "Ctrl+I", "Cmd+I");
    pub const INLINE_CODE: MenuShortcut = MenuShortcut::new("secondary-e", "Ctrl+E", "Cmd+E");
    pub const INSERT_LINK: MenuShortcut = MenuShortcut::new("secondary-k", "Ctrl+K", "Cmd+K");
    pub const INSERT_IMAGE: MenuShortcut =
        MenuShortcut::new("secondary-shift-i", "Ctrl+Shift+I", "Cmd+Shift+I");
    pub const HEADING_1: MenuShortcut = MenuShortcut::new("secondary-1", "Ctrl+1", "Cmd+1");
    pub const HEADING_2: MenuShortcut = MenuShortcut::new("secondary-2", "Ctrl+2", "Cmd+2");
    pub const HEADING_3: MenuShortcut = MenuShortcut::new("secondary-3", "Ctrl+3", "Cmd+3");
    pub const HEADING_4: MenuShortcut = MenuShortcut::new("secondary-4", "Ctrl+4", "Cmd+4");
    pub const HEADING_5: MenuShortcut = MenuShortcut::new("secondary-5", "Ctrl+5", "Cmd+5");
    pub const HEADING_6: MenuShortcut = MenuShortcut::new("secondary-6", "Ctrl+6", "Cmd+6");
    pub const FORMAT_TABLE: MenuShortcut =
        MenuShortcut::new("secondary-shift-m", "Ctrl+Shift+M", "Cmd+Shift+M");
    pub const TABLE_ADD_ROW: MenuShortcut =
        MenuShortcut::new("secondary-alt-enter", "Ctrl+Alt+Enter", "Cmd+Option+Enter");
    pub const TABLE_DELETE_ROW: MenuShortcut = MenuShortcut::new(
        "secondary-alt-backspace",
        "Ctrl+Alt+Backspace",
        "Cmd+Option+Backspace",
    );
    pub const TABLE_MOVE_ROW_UP: MenuShortcut =
        MenuShortcut::new("secondary-alt-up", "Ctrl+Alt+Up", "Cmd+Option+Up");
    pub const TABLE_MOVE_ROW_DOWN: MenuShortcut =
        MenuShortcut::new("secondary-alt-down", "Ctrl+Alt+Down", "Cmd+Option+Down");
    pub const TABLE_ADD_COLUMN: MenuShortcut =
        MenuShortcut::new("secondary-alt-right", "Ctrl+Alt+Right", "Cmd+Option+Right");
    pub const TABLE_DELETE_COLUMN: MenuShortcut =
        MenuShortcut::new("secondary-alt-left", "Ctrl+Alt+Left", "Cmd+Option+Left");

    pub const EXPORT_HTML: MenuShortcut =
        MenuShortcut::new("secondary-shift-h", "Ctrl+Shift+H", "Cmd+Shift+H");
    pub const EXPORT_PLAIN_HTML: MenuShortcut = MenuShortcut::new(
        "secondary-alt-shift-h",
        "Ctrl+Alt+Shift+H",
        "Cmd+Option+Shift+H",
    );
    pub const EXPORT_PDF: MenuShortcut =
        MenuShortcut::new("secondary-shift-p", "Ctrl+Shift+P", "Cmd+Shift+P");
    pub const EXPORT_LATEX: MenuShortcut =
        MenuShortcut::new("secondary-shift-l", "Ctrl+Shift+L", "Cmd+Shift+L");
    pub const EXPORT_DOCX: MenuShortcut =
        MenuShortcut::new("secondary-shift-d", "Ctrl+Shift+D", "Cmd+Shift+D");
    pub const EXPORT_PNG: MenuShortcut =
        MenuShortcut::new("secondary-shift-g", "Ctrl+Shift+G", "Cmd+Shift+G");
    pub const EXPORT_JPEG: MenuShortcut = MenuShortcut::new(
        "secondary-alt-shift-g",
        "Ctrl+Alt+Shift+G",
        "Cmd+Option+Shift+G",
    );

    pub const SHOW_SHORTCUTS: MenuShortcut = MenuShortcut::new("f1", "F1", "F1");
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
            // Chinese labels (文件/編輯/檢視/格式/匯出/說明) — narrower. Both
            // Simplified and Traditional share this column: the glyph widths
            // are nearly identical, so the hand-tuned offsets apply to both.
            (Language::ZhHans | Language::ZhHant, AppMenu::File) => px(8.),
            (Language::ZhHans | Language::ZhHant, AppMenu::Edit) => px(50.),
            (Language::ZhHans | Language::ZhHant, AppMenu::View) => px(92.),
            (Language::ZhHans | Language::ZhHant, AppMenu::Format) => px(134.),
            (Language::ZhHans | Language::ZhHant, AppMenu::Export) => px(178.),
            (Language::ZhHans | Language::ZhHant, AppMenu::Help) => px(222.),
        }
    }

    fn dropdown_width(self, _language: Language) -> Pixels {
        // Keep enough room for the longest localized label plus the menu's
        // right-aligned platform shortcut. Left offsets remain language-tuned
        // independently above, so widening a dropdown does not move its title.
        match self {
            AppMenu::File => px(280.),
            AppMenu::Edit => px(264.),
            AppMenu::View => px(304.),
            AppMenu::Format => px(344.),
            AppMenu::Export => px(288.),
            AppMenu::Help => px(236.),
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
    HtmlText,
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
            Self::HtmlText => (5, 0, 0),
            Self::TableCell { row, col } => (6, row, col),
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
mod appearance;
mod application;
mod bootstrap;
mod diagram;
mod documents;
mod editing;
mod editor_element;
mod network;
mod preview;
mod root_view;
mod save_dialog;
mod search;
mod state;
mod workspace;

#[cfg(test)]
mod tests;

use bootstrap::install_menus;
use diagram::*;
use editor_element::EditorElement;
use preview::*;
use root_view::*;
use save_dialog::*;
use state::*;

pub(super) fn run() {
    bootstrap::run();
}

type HighlightCache = RefCell<HashMap<(Option<String>, String), Rc<Vec<Vec<HighlightedSpan>>>>>;

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
    /// Modal Help -> Keyboard Shortcuts panel state. This is transient UI state
    /// and is deliberately not persisted with editor preferences.
    shortcut_panel_open: bool,
    shortcut_panel_focus: FocusHandle,
    shortcut_platform: ShortcutPlatform,
    shortcut_category: ShortcutCategory,
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
    /// Set when a replacement workspace root still needs its first successful
    /// scan to seed the one-level default tree view.
    file_tree_needs_initial_collapse: bool,
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
    highlight_cache: HighlightCache,
    /// Shared across tabs and frames; pending entries are never evicted.
    diagram_cache: DiagramCache,
    /// Active interface language. Persisted via `AppPreferences::language`.
    language: Language,
}
