use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
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
    App, Application, Bounds, ClickEvent, ClipboardEntry, ClipboardItem, Context, CursorStyle,
    DefiniteLength, DispatchPhase, Div, DragMoveEvent, Element, ElementId, ElementInputHandler,
    Empty, Entity, EntityInputHandler, ExternalPaths, FocusHandle, Focusable, Font, FontFallbacks,
    FontFeatures, FontStyle, FontWeight, GlobalElementId, HighlightStyle, Hitbox, HitboxBehavior,
    ImageFormat, ImageSource, KeyBinding, KeyDownEvent, LayoutId, ListAlignment, ListState, Menu,
    MenuItem, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad,
    PathPromptOptions, Pixels, Point, PromptButton, PromptLevel, RenderImage, Rgba, ScrollHandle,
    SharedString, Size, Stateful, StrikethroughStyle, Style, StyledText, TextLayout, TextRun,
    Timer, TitlebarOptions, UTF16Selection, UnderlineStyle, Window, WindowBounds, WindowOptions,
    WrappedLine, actions, anchored, canvas, div, fill, font, img, list, point, px, rgb, rgba, size,
};
use markion::{
    AlertKind, AppPreferences, AutoSavePreferences, BlockEdit, BlockEditError, BlockPlacement,
    BlockTarget, BlockTransform, CheckedMutation, DEFAULT_CODE_FONT_FAMILY,
    DEFAULT_EDITOR_FONT_SIZE, DEFAULT_HEADING_MENU_MAX_LEVEL, DEFAULT_RENDERED_FONT_SIZE,
    DiskIdentity, DiskState, DocumentInstanceId, DocxImagePolicy, DocxPageSize,
    EXTENDED_HEADING_MENU_MAX_LEVEL, ExportBackendPreference, ExportFormat, ExportPreferences,
    ExternalCheckOutcome, FileTree, FileTreeEntry, FileTreeEntryKind, HighlightKind,
    HighlightedSpan, HtmlAlign, HtmlImgLength, HtmlListMarker, HtmlPreviewPart, HtmlTableGrid,
    ImageAlignment, ImagePresentation, InlineSpan, InlineStyle, Language, MAX_AUTO_SAVE_DELAY_SECS,
    MAX_EDITOR_FONT_SIZE, MAX_PARAGRAPH_SPACING, MAX_RENDERED_FONT_SIZE, MIN_AUTO_SAVE_DELAY_SECS,
    MIN_EDITOR_FONT_SIZE, MIN_PARAGRAPH_SPACING, MIN_RENDERED_FONT_SIZE, MarkdownDocument,
    MarkdownFormat, MathLayoutStyle, Msg, MutationOrigin, MutationReceipt, P0Msg, P1Msg,
    PdfPageSize, PreviewBlock, RecoveryInventoryEntry, RecoverySourceState, RichText,
    SYSTEM_UI_FONT_FAMILY, SearchMatchRange, SearchOptions, SearchPattern, SessionState,
    ShortcutCategory, ShortcutPlatform, SidebarTab, SlashCommand, SlashQuery, TableEdit,
    ThemeColors, ThemeDefinition, ThemeFonts, ViewMode, VisualBlock, VisualBlockEditor,
    VisualBlockId, VisualBlockKind, VisualCaretAffinity, VisualEditorField, VisualEditorFieldKind,
    VisualHtmlImage, VisualNavigationTarget, VisualProjection, VisualQuoteGroupEdge,
    VisualSourceIslandKind, adjacent_reorder_target, backend_status_msg, block_can_reorder_at,
    block_can_transform_at, build_publishing_snapshot, build_visual_projection,
    build_visual_projection_with_marked_range, builtin_diagram_registry, builtin_theme_definitions,
    bundled_resource_path, check_path_state, default_preferences_path, default_recovery_dir,
    default_session_path, default_themes_dir, delete_block, delete_recovery_file,
    diagram_backend_id, duplicate_block, highlight_code, html_preview_parts,
    html_preview_plain_text, html_table_column_weights, html_table_grid_line_end,
    html_table_row_has_visible_header, image_extension_supported, import_image_bytes,
    import_image_file, inline_image_at, inline_link_at, inspect_recovery_files, is_markdown_path,
    is_text_path, list_theme_definitions, load_app_preferences, load_recovery_file,
    load_session_state, markdown_reference, normalize_auto_save_delay_secs,
    normalize_editor_font_size, normalize_heading_menu_max_level, normalize_paragraph_spacing,
    normalize_rendered_font_size, p0_t, p0_tf, p1_t, p1_tf, pandoc_available, read_document_source,
    reorder_block, resolve_font_family, resolve_html_img_display_size, save_app_preferences,
    save_session_state, save_text_snapshot, save_theme_definition, serialize_inline_image,
    serialize_inline_link, shortcut_catalog, sidebar_tab_label, slash_command_edit, slash_query_at,
    t, table_column_flex_weights, tf, title_from_path, transform_block, validate_block_target,
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
        SearchPreviousOrNewline,
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
        ShowVisualBlockContextMenu,
        NewDocument,
        OpenDocument,
        OpenFolder,
        ClearRecentFiles,
        SaveDocument,
        SaveDocumentAs,
        ExportHtml,
        ExportPlainHtml,
        ExportPdf,
        ExportLatex,
        ExportDocx,
        ExportPng,
        ExportJpeg,
        PublishWechat,
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
        ShowMarkdownReference,
        ShowPreferences,
        ResetPreferences,
        CheckForUpdates,
        ReportIssue,
        OpenOnlineDocs,
        AboutMarkion,
        Quit,
        NewTab,
        OpenInNewTab,
        CloseTab,
        NextTab,
        PrevTab,
        /// Developer-facing: write a per-site retained-memory report to the
        /// diagnostic log. Not advertised in menu chrome.
        ReportMemory,
    ]
);

const MARKION_APP_ID: &str = "dev.markion.app";
const MARKION_WINDOW_TITLE: &str = "Markion";

const MAX_HISTORY_LEN: usize = 200;
const MARKION_PROJECT_WEBSITE_URL: &str = "https://markion.app";
const GITHUB_REPO_URL: &str = "https://github.com/willmove/markion";
const GITHUB_ISSUES_URL: &str = "https://github.com/willmove/markion/issues/new";
const GITHUB_DOCS_URL: &str = "https://github.com/willmove/markion#readme";
const KENHUANG_MARKDOWN_TUTORIAL_ZH_URL: &str = "https://kenhuang.com/markdown/";
const KENHUANG_MARKDOWN_TUTORIAL_EN_URL: &str = "https://kenhuang.com/en/markdown/";

fn kenhuang_markdown_tutorial_url(language: Language) -> &'static str {
    match language {
        Language::ZhHans | Language::ZhHant => KENHUANG_MARKDOWN_TUTORIAL_ZH_URL,
        _ => KENHUANG_MARKDOWN_TUTORIAL_EN_URL,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AboutLink {
    ProjectWebsite,
    GithubRepository,
}

impl AboutLink {
    const ALL: [Self; 2] = [Self::ProjectWebsite, Self::GithubRepository];

    const fn label(self) -> Msg {
        match self {
            Self::ProjectWebsite => Msg::DialogAboutProjectWebsite,
            Self::GithubRepository => Msg::DialogAboutGithub,
        }
    }

    const fn url(self) -> &'static str {
        match self {
            Self::ProjectWebsite => MARKION_PROJECT_WEBSITE_URL,
            Self::GithubRepository => GITHUB_REPO_URL,
        }
    }

    const fn row_selector(self) -> &'static str {
        match self {
            Self::ProjectWebsite => "about-project-website-row",
            Self::GithubRepository => "about-github-row",
        }
    }

    const fn link_selector(self) -> &'static str {
        match self {
            Self::ProjectWebsite => "about-project-website-link",
            Self::GithubRepository => "about-github-link",
        }
    }
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

fn menu_after_hover(active: Option<AppMenu>, hovered: AppMenu) -> Option<AppMenu> {
    active.map(|_| hovered)
}

/// One source of truth for a customizable keyboard action: its stable id
/// (the `config.toml` `[shortcuts]` key), default GPUI binding, and the
/// curated text shown beside its in-window menu item and in the shortcut
/// reference. Explicit platform labels keep GPUI's internal `secondary`
/// modifier out of user-facing chrome; overridden bindings are rendered
/// through `markion::keystroke::format_keystroke_label` instead.
///
/// `binding` is `None` for factory-unbound actions (currently
/// `show-shortcuts`): they stay in the registry for Preferences capture but
/// are not installed in the keymap unless a valid override exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MenuShortcut {
    id: &'static str,
    binding: Option<&'static str>,
    windows_linux: &'static str,
    macos: &'static str,
}

impl MenuShortcut {
    const fn new(
        id: &'static str,
        binding: &'static str,
        windows_linux: &'static str,
        macos: &'static str,
    ) -> Self {
        Self {
            id,
            binding: Some(binding),
            windows_linux,
            macos,
        }
    }

    const fn unbound(id: &'static str) -> Self {
        Self {
            id,
            binding: None,
            windows_linux: "",
            macos: "",
        }
    }

    const fn label(self, platform: ShortcutPlatform) -> &'static str {
        match platform {
            ShortcutPlatform::WindowsLinux => self.windows_linux,
            ShortcutPlatform::MacOS => self.macos,
        }
    }

    /// The binding that should dispatch this action: a valid stored override
    /// when present, otherwise the factory default. `None` means the action
    /// has no keystroke until the user assigns one.
    fn effective_binding<'a>(&self, overrides: &'a BTreeMap<String, String>) -> Option<&'a str> {
        if let Some(binding) = overrides.get(self.id).filter(|binding| {
            markion::keystroke::KeystrokeParts::parse(binding, ShortcutPlatform::current())
                .is_some()
                && gpui::Keystroke::parse(binding).is_ok()
        }) {
            return Some(binding.as_str());
        }
        self.binding
    }

    /// The label shown in menus and the shortcut reference: the curated
    /// default label, or a formatted rendering of the override. Empty when
    /// the action is factory-unbound and has no override.
    fn effective_label(
        &self,
        overrides: &BTreeMap<String, String>,
        platform: ShortcutPlatform,
    ) -> String {
        if let Some(binding) = overrides.get(self.id).filter(|binding| {
            markion::keystroke::KeystrokeParts::parse(binding, platform).is_some()
        }) {
            markion::keystroke::format_keystroke_label(binding, platform)
        } else {
            self.label(platform).to_string()
        }
    }
}

/// Look up a registry entry by its stable action id.
fn shortcut_by_id(id: &str) -> Option<&'static MenuShortcut> {
    menu_shortcuts::ALL
        .iter()
        .find(|shortcut| shortcut.id == id)
}

/// Shared descriptors for actions that appear in the six application menus
/// (plus the file-tree search focus shortcut, which lives in the same
/// customizable registry). Unbound menu actions intentionally have no entry
/// in this module.
mod menu_shortcuts {
    use super::MenuShortcut;

    pub const NEW_DOCUMENT: MenuShortcut =
        MenuShortcut::new("new-document", "secondary-n", "Ctrl+N", "Cmd+N");
    pub const OPEN_DOCUMENT: MenuShortcut =
        MenuShortcut::new("open-document", "secondary-o", "Ctrl+O", "Cmd+O");
    pub const OPEN_FOLDER: MenuShortcut = MenuShortcut::new(
        "open-folder",
        "secondary-shift-o",
        "Ctrl+Shift+O",
        "Cmd+Shift+O",
    );
    pub const SAVE_DOCUMENT: MenuShortcut =
        MenuShortcut::new("save-document", "secondary-s", "Ctrl+S", "Cmd+S");
    pub const SAVE_DOCUMENT_AS: MenuShortcut = MenuShortcut::new(
        "save-document-as",
        "secondary-shift-s",
        "Ctrl+Shift+S",
        "Cmd+Shift+S",
    );
    pub const OPEN_IN_NEW_TAB: MenuShortcut =
        MenuShortcut::new("open-in-new-tab", "secondary-t", "Ctrl+T", "Cmd+T");
    pub const NEW_TAB: MenuShortcut = MenuShortcut::new(
        "new-tab",
        "secondary-shift-n",
        "Ctrl+Shift+N",
        "Cmd+Shift+N",
    );
    pub const CLOSE_TAB: MenuShortcut =
        MenuShortcut::new("close-tab", "secondary-w", "Ctrl+W", "Cmd+W");
    pub const NEXT_TAB: MenuShortcut =
        MenuShortcut::new("next-tab", "ctrl-tab", "Ctrl+Tab", "Ctrl+Tab");
    pub const PREV_TAB: MenuShortcut = MenuShortcut::new(
        "prev-tab",
        "ctrl-shift-tab",
        "Ctrl+Shift+Tab",
        "Ctrl+Shift+Tab",
    );
    pub const SHOW_PREFERENCES: MenuShortcut =
        MenuShortcut::new("show-preferences", "secondary-comma", "Ctrl+,", "Cmd+,");
    pub const QUIT: MenuShortcut = MenuShortcut::new("quit", "secondary-q", "Ctrl+Q", "Cmd+Q");

    pub const UNDO: MenuShortcut = MenuShortcut::new("undo", "secondary-z", "Ctrl+Z", "Cmd+Z");
    pub const REDO: MenuShortcut = MenuShortcut::new("redo", "secondary-y", "Ctrl+Y", "Cmd+Y");
    pub const COPY: MenuShortcut = MenuShortcut::new("copy", "secondary-c", "Ctrl+C", "Cmd+C");
    pub const CUT: MenuShortcut = MenuShortcut::new("cut", "secondary-x", "Ctrl+X", "Cmd+X");
    pub const PASTE: MenuShortcut = MenuShortcut::new("paste", "secondary-v", "Ctrl+V", "Cmd+V");
    pub const SELECT_ALL: MenuShortcut =
        MenuShortcut::new("select-all", "secondary-a", "Ctrl+A", "Cmd+A");

    pub const TOGGLE_VIEW_MODE: MenuShortcut = MenuShortcut::new(
        "toggle-view-mode",
        "secondary-shift-v",
        "Ctrl+Shift+V",
        "Cmd+Shift+V",
    );
    pub const SET_EDIT_MODE: MenuShortcut =
        MenuShortcut::new("set-edit-mode", "secondary-/", "Ctrl+/", "Cmd+/");
    pub const SET_VISUAL_EDIT_MODE: MenuShortcut =
        MenuShortcut::new("set-visual-edit-mode", "secondary-e", "Ctrl+E", "Cmd+E");
    pub const SET_SPLIT_PREVIEW_MODE: MenuShortcut =
        MenuShortcut::new("set-split-preview-mode", "secondary-p", "Ctrl+P", "Cmd+P");
    pub const SET_READ_MODE: MenuShortcut =
        MenuShortcut::new("set-read-mode", "secondary-r", "Ctrl+R", "Cmd+R");
    pub const TOGGLE_SIDEBAR: MenuShortcut = MenuShortcut::new(
        "toggle-sidebar",
        "secondary-shift-b",
        "Ctrl+Shift+B",
        "Cmd+Shift+B",
    );
    pub const TOGGLE_FILE_TREE: MenuShortcut = MenuShortcut::new(
        "toggle-file-tree",
        "secondary-shift-f",
        "Ctrl+Shift+F",
        "Cmd+Shift+F",
    );
    pub const TOGGLE_OUTLINE: MenuShortcut = MenuShortcut::new("toggle-outline", "f6", "F6", "F6");
    pub const TOGGLE_FOCUS_MODE: MenuShortcut =
        MenuShortcut::new("toggle-focus-mode", "f7", "F7", "F7");
    pub const TOGGLE_TYPEWRITER_MODE: MenuShortcut =
        MenuShortcut::new("toggle-typewriter-mode", "f8", "F8", "F8");
    pub const TOGGLE_CODE_LINE_NUMBERS: MenuShortcut = MenuShortcut::new(
        "toggle-code-line-numbers",
        "secondary-shift-4",
        "Ctrl+Shift+4",
        "Cmd+Shift+4",
    );
    pub const SHOW_FIND: MenuShortcut =
        MenuShortcut::new("show-find", "secondary-f", "Ctrl+F", "Cmd+F");
    pub const SHOW_REPLACE: MenuShortcut =
        MenuShortcut::new("show-replace", "secondary-h", "Ctrl+H", "Cmd+H");
    pub const FIND_NEXT: MenuShortcut = MenuShortcut::new("find-next", "f3", "F3", "F3");
    pub const FIND_PREVIOUS: MenuShortcut =
        MenuShortcut::new("find-previous", "shift-f3", "Shift+F3", "Shift+F3");
    pub const CYCLE_THEME: MenuShortcut = MenuShortcut::new(
        "cycle-theme",
        "secondary-shift-t",
        "Ctrl+Shift+T",
        "Cmd+Shift+T",
    );
    pub const FOCUS_FILE_TREE_SEARCH: MenuShortcut = MenuShortcut::new(
        "focus-file-tree-search",
        "secondary-alt-f",
        "Ctrl+Alt+F",
        "Cmd+Option+F",
    );

    pub const BOLD: MenuShortcut = MenuShortcut::new("bold", "secondary-b", "Ctrl+B", "Cmd+B");
    pub const ITALIC: MenuShortcut = MenuShortcut::new("italic", "secondary-i", "Ctrl+I", "Cmd+I");
    pub const INLINE_CODE: MenuShortcut = MenuShortcut::new(
        "inline-code",
        "secondary-shift-`",
        "Ctrl+Shift+`",
        "Cmd+Shift+`",
    );
    pub const INSERT_LINK: MenuShortcut =
        MenuShortcut::new("insert-link", "secondary-k", "Ctrl+K", "Cmd+K");
    pub const INSERT_IMAGE: MenuShortcut = MenuShortcut::new(
        "insert-image",
        "secondary-shift-i",
        "Ctrl+Shift+I",
        "Cmd+Shift+I",
    );
    pub const HEADING_1: MenuShortcut =
        MenuShortcut::new("heading-1", "secondary-1", "Ctrl+1", "Cmd+1");
    pub const HEADING_2: MenuShortcut =
        MenuShortcut::new("heading-2", "secondary-2", "Ctrl+2", "Cmd+2");
    pub const HEADING_3: MenuShortcut =
        MenuShortcut::new("heading-3", "secondary-3", "Ctrl+3", "Cmd+3");
    pub const HEADING_4: MenuShortcut =
        MenuShortcut::new("heading-4", "secondary-4", "Ctrl+4", "Cmd+4");
    pub const HEADING_5: MenuShortcut =
        MenuShortcut::new("heading-5", "secondary-5", "Ctrl+5", "Cmd+5");
    pub const HEADING_6: MenuShortcut =
        MenuShortcut::new("heading-6", "secondary-6", "Ctrl+6", "Cmd+6");
    pub const UNORDERED_LIST: MenuShortcut = MenuShortcut::new(
        "unordered-list",
        "secondary-shift-]",
        "Ctrl+Shift+]",
        "Cmd+Shift+]",
    );
    pub const ORDERED_LIST: MenuShortcut = MenuShortcut::new(
        "ordered-list",
        "secondary-shift-[",
        "Ctrl+Shift+[",
        "Cmd+Shift+[",
    );
    pub const TASK_LIST: MenuShortcut = MenuShortcut::new(
        "task-list",
        "secondary-shift-x",
        "Ctrl+Shift+X",
        "Cmd+Shift+X",
    );
    pub const BLOCK_QUOTE: MenuShortcut = MenuShortcut::new(
        "block-quote",
        "secondary-shift-q",
        "Ctrl+Shift+Q",
        "Cmd+Shift+Q",
    );
    pub const CODE_FENCE: MenuShortcut = MenuShortcut::new(
        "code-fence",
        "secondary-shift-k",
        "Ctrl+Shift+K",
        "Cmd+Shift+K",
    );
    pub const FORMAT_TABLE: MenuShortcut = MenuShortcut::new(
        "format-table",
        "secondary-shift-m",
        "Ctrl+Shift+M",
        "Cmd+Shift+M",
    );
    pub const TABLE_ADD_ROW: MenuShortcut = MenuShortcut::new(
        "table-add-row",
        "secondary-alt-enter",
        "Ctrl+Alt+Enter",
        "Cmd+Option+Enter",
    );
    pub const TABLE_DELETE_ROW: MenuShortcut = MenuShortcut::new(
        "table-delete-row",
        "secondary-alt-backspace",
        "Ctrl+Alt+Backspace",
        "Cmd+Option+Backspace",
    );
    pub const TABLE_MOVE_ROW_UP: MenuShortcut = MenuShortcut::new(
        "table-move-row-up",
        "secondary-alt-up",
        "Ctrl+Alt+Up",
        "Cmd+Option+Up",
    );
    pub const TABLE_MOVE_ROW_DOWN: MenuShortcut = MenuShortcut::new(
        "table-move-row-down",
        "secondary-alt-down",
        "Ctrl+Alt+Down",
        "Cmd+Option+Down",
    );
    pub const TABLE_ADD_COLUMN: MenuShortcut = MenuShortcut::new(
        "table-add-column",
        "secondary-alt-right",
        "Ctrl+Alt+Right",
        "Cmd+Option+Right",
    );
    pub const TABLE_DELETE_COLUMN: MenuShortcut = MenuShortcut::new(
        "table-delete-column",
        "secondary-alt-left",
        "Ctrl+Alt+Left",
        "Cmd+Option+Left",
    );

    pub const EXPORT_HTML: MenuShortcut = MenuShortcut::new(
        "export-html",
        "secondary-shift-h",
        "Ctrl+Shift+H",
        "Cmd+Shift+H",
    );
    pub const EXPORT_PLAIN_HTML: MenuShortcut = MenuShortcut::new(
        "export-plain-html",
        "secondary-alt-shift-h",
        "Ctrl+Alt+Shift+H",
        "Cmd+Option+Shift+H",
    );
    pub const EXPORT_PDF: MenuShortcut = MenuShortcut::new(
        "export-pdf",
        "secondary-shift-p",
        "Ctrl+Shift+P",
        "Cmd+Shift+P",
    );
    pub const EXPORT_LATEX: MenuShortcut = MenuShortcut::new(
        "export-latex",
        "secondary-shift-l",
        "Ctrl+Shift+L",
        "Cmd+Shift+L",
    );
    pub const EXPORT_DOCX: MenuShortcut = MenuShortcut::new(
        "export-docx",
        "secondary-shift-d",
        "Ctrl+Shift+D",
        "Cmd+Shift+D",
    );
    pub const EXPORT_PNG: MenuShortcut = MenuShortcut::new(
        "export-png",
        "secondary-shift-g",
        "Ctrl+Shift+G",
        "Cmd+Shift+G",
    );
    pub const EXPORT_JPEG: MenuShortcut = MenuShortcut::new(
        "export-jpeg",
        "secondary-alt-shift-g",
        "Ctrl+Alt+Shift+G",
        "Cmd+Option+Shift+G",
    );

    pub const SHOW_MARKDOWN_REFERENCE: MenuShortcut =
        MenuShortcut::new("show-markdown-reference", "f1", "F1", "F1");
    /// Factory-unbound: Preferences → Shortcuts remains reachable from the
    /// Preferences panel; users may assign a keystroke later.
    pub const SHOW_SHORTCUTS: MenuShortcut = MenuShortcut::unbound("show-shortcuts");

    /// Every customizable action, in registry order. Used for rebinding,
    /// conflict detection, and validating stored overrides.
    pub const ALL: &[MenuShortcut] = &[
        NEW_DOCUMENT,
        OPEN_DOCUMENT,
        OPEN_FOLDER,
        SAVE_DOCUMENT,
        SAVE_DOCUMENT_AS,
        OPEN_IN_NEW_TAB,
        NEW_TAB,
        CLOSE_TAB,
        NEXT_TAB,
        PREV_TAB,
        SHOW_PREFERENCES,
        QUIT,
        UNDO,
        REDO,
        COPY,
        CUT,
        PASTE,
        SELECT_ALL,
        TOGGLE_VIEW_MODE,
        SET_EDIT_MODE,
        SET_VISUAL_EDIT_MODE,
        SET_SPLIT_PREVIEW_MODE,
        SET_READ_MODE,
        TOGGLE_SIDEBAR,
        TOGGLE_FILE_TREE,
        TOGGLE_OUTLINE,
        TOGGLE_FOCUS_MODE,
        TOGGLE_TYPEWRITER_MODE,
        TOGGLE_CODE_LINE_NUMBERS,
        SHOW_FIND,
        SHOW_REPLACE,
        FIND_NEXT,
        FIND_PREVIOUS,
        CYCLE_THEME,
        FOCUS_FILE_TREE_SEARCH,
        BOLD,
        ITALIC,
        INLINE_CODE,
        INSERT_LINK,
        INSERT_IMAGE,
        HEADING_1,
        HEADING_2,
        HEADING_3,
        HEADING_4,
        HEADING_5,
        HEADING_6,
        UNORDERED_LIST,
        ORDERED_LIST,
        TASK_LIST,
        BLOCK_QUOTE,
        CODE_FENCE,
        FORMAT_TABLE,
        TABLE_ADD_ROW,
        TABLE_DELETE_ROW,
        TABLE_MOVE_ROW_UP,
        TABLE_MOVE_ROW_DOWN,
        TABLE_ADD_COLUMN,
        TABLE_DELETE_COLUMN,
        EXPORT_HTML,
        EXPORT_PLAIN_HTML,
        EXPORT_PDF,
        EXPORT_LATEX,
        EXPORT_DOCX,
        EXPORT_PNG,
        EXPORT_JPEG,
        SHOW_MARKDOWN_REFERENCE,
        SHOW_SHORTCUTS,
    ];
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
            AppMenu::Help => px(280.),
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

/// In-flight inline name editor for a file-tree create/rename action. The
/// buffer is the text the user is editing; on Enter the kind decides which
/// `FileTree` operation runs, and on Escape the editor is dropped without
/// touching the filesystem. The editor reuses the app's redirected-text-input
/// path (the same one the search field and file-tree filter use) so IME
/// composition is handled identically. `cursor` is the caret position as a
/// byte offset into `buffer`; `anchor` is the other end of the selection, so
/// the selection spans `cursor..anchor` in either order and is collapsed
/// when the two are equal.
#[derive(Clone, Debug)]
struct PendingNameInput {
    kind: PendingNameKind,
    /// Directory the new entry is created in (create), or the parent of the
    /// entry being renamed (rename).
    parent: PathBuf,
    /// The entry being renamed; `None` for create actions.
    target: Option<PathBuf>,
    buffer: String,
    /// Caret position (byte offset into `buffer`, always on a char boundary).
    cursor: usize,
    /// Selection anchor (byte offset). Shift+arrows move the cursor while the
    /// anchor stays put; plain arrows collapse both.
    anchor: usize,
}

impl PendingNameInput {
    /// Build an editor for `kind` with `prefill` as the initial buffer. For
    /// renames the base name is selected and the extension preserved, so
    /// typing replaces the base name in one stroke; other kinds select the
    /// whole prefilled name.
    fn new(kind: PendingNameKind, parent: PathBuf, target: Option<PathBuf>, prefill: &str) -> Self {
        let selected_len = match kind {
            PendingNameKind::Rename => base_name_len(prefill),
            PendingNameKind::CreateFile | PendingNameKind::CreateFolder => prefill.len(),
        };
        Self {
            kind,
            parent,
            target,
            buffer: prefill.to_string(),
            cursor: selected_len,
            anchor: 0,
        }
    }

    /// The selection as an ordered, boundary-clamped byte range.
    fn selection(&self) -> Range<usize> {
        let len = self.buffer.len();
        let (start, end) = if self.cursor <= self.anchor {
            (self.cursor, self.anchor)
        } else {
            (self.anchor, self.cursor)
        };
        start.min(len)..end.min(len)
    }

    /// Clamp cursor/anchor back onto char boundaries inside the buffer (the
    /// buffer shrinks on edits; stale offsets must never panic a slice).
    fn clamp_to_boundaries(&mut self) {
        let len = self.buffer.len();
        self.cursor = self.cursor.min(len);
        while self.cursor > 0 && !self.buffer.is_char_boundary(self.cursor) {
            self.cursor -= 1;
        }
        self.anchor = self.anchor.min(len);
        while self.anchor > 0 && !self.buffer.is_char_boundary(self.anchor) {
            self.anchor -= 1;
        }
    }
}

/// Length in bytes of the base-name portion of `name` — everything before the
/// extension separator. The final `.` of the last path component starts the
/// extension, unless it is the first character (dotfiles like `.gitignore`
/// have no separate base name).
fn base_name_len(name: &str) -> usize {
    let component = name.rsplit(['/', '\\']).next().unwrap_or(name);
    match component.rfind('.') {
        Some(dot) if dot > 0 => name.len() - (component.len() - dot),
        _ => name.len(),
    }
}

/// Byte offset of the previous char boundary before `offset` in `s`.
fn previous_name_boundary(s: &str, offset: usize) -> usize {
    let mut i = offset.min(s.len());
    if i == 0 {
        return 0;
    }
    i -= 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Byte offset of the next char boundary after `offset` in `s`.
fn next_name_boundary(s: &str, offset: usize) -> usize {
    let mut i = offset.min(s.len());
    if i >= s.len() {
        return s.len();
    }
    i += 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Directions the inline name editor's caret can move. Shared by the Left /
/// Right / Home / End / Select* action handlers so every redirected key
/// mutates the name buffer instead of the document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NameCaretMove {
    Left,
    Right,
    Home,
    End,
    SelectLeft,
    SelectRight,
    SelectAll,
}

#[derive(Clone, Debug)]
struct PendingImageInput {
    stem: String,
    extension: String,
    bytes: Vec<u8>,
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

fn tab_context_action_label(action: TabContextAction) -> Msg {
    match action {
        TabContextAction::CloseTab => Msg::ItemTabClose,
        TabContextAction::CloseOthers => Msg::ItemTabCloseOthers,
        TabContextAction::CloseToTheRight => Msg::ItemTabCloseToTheRight,
        TabContextAction::Rename => Msg::ItemTabRename,
        TabContextAction::CopyPath => Msg::ItemTabCopyPath,
        TabContextAction::RevealInFileManager => Msg::ItemTabRevealInFileManager,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchField {
    Find,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchOverlayControl {
    FindField,
    ReplaceField,
    Previous,
    Next,
    MatchCase,
    Regex,
    ReplaceCurrent,
    ReplaceAll,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchCaretMove {
    Left,
    Right,
    Home,
    End,
    SelectLeft,
    SelectRight,
    SelectAll,
}

/// UTF-8-safe state for one single-line search overlay field.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SearchFieldState {
    buffer: String,
    cursor: usize,
    anchor: usize,
    marked_range: Option<Range<usize>>,
}

impl SearchFieldState {
    fn new(buffer: impl Into<String>) -> Self {
        let buffer = buffer.into();
        let end = buffer.len();
        Self {
            buffer,
            cursor: end,
            anchor: end,
            marked_range: None,
        }
    }

    fn selection(&self) -> Range<usize> {
        self.anchor.min(self.cursor)..self.anchor.max(self.cursor)
    }

    fn clamp_to_boundaries(&mut self) {
        self.cursor = clamp_search_boundary(&self.buffer, self.cursor);
        self.anchor = clamp_search_boundary(&self.buffer, self.anchor);
        self.marked_range = self.marked_range.take().map(|range| {
            clamp_search_boundary(&self.buffer, range.start)
                ..clamp_search_boundary(&self.buffer, range.end)
        });
    }

    fn set_text(&mut self, text: impl Into<String>) {
        self.buffer = text.into();
        self.cursor = self.buffer.len();
        self.anchor = self.cursor;
        self.marked_range = None;
    }

    fn move_caret(&mut self, movement: SearchCaretMove) {
        self.clamp_to_boundaries();
        let selection = self.selection();
        match movement {
            SearchCaretMove::Left => {
                self.cursor = if selection.is_empty() {
                    previous_search_boundary(&self.buffer, self.cursor)
                } else {
                    selection.start
                };
                self.anchor = self.cursor;
            }
            SearchCaretMove::Right => {
                self.cursor = if selection.is_empty() {
                    next_search_boundary(&self.buffer, self.cursor)
                } else {
                    selection.end
                };
                self.anchor = self.cursor;
            }
            SearchCaretMove::Home => {
                self.cursor = 0;
                self.anchor = 0;
            }
            SearchCaretMove::End => {
                self.cursor = self.buffer.len();
                self.anchor = self.cursor;
            }
            SearchCaretMove::SelectLeft => {
                self.cursor = previous_search_boundary(&self.buffer, self.cursor);
            }
            SearchCaretMove::SelectRight => {
                self.cursor = next_search_boundary(&self.buffer, self.cursor);
            }
            SearchCaretMove::SelectAll => {
                self.anchor = 0;
                self.cursor = self.buffer.len();
            }
        }
        self.marked_range = None;
    }

    fn replace_selection(&mut self, text: &str, marked: bool) {
        self.clamp_to_boundaries();
        let range = self
            .marked_range
            .clone()
            .unwrap_or_else(|| self.selection());
        self.buffer.replace_range(range.clone(), text);
        self.cursor = range.start + text.len();
        self.anchor = self.cursor;
        self.marked_range = marked.then_some(range.start..self.cursor);
    }

    fn backspace(&mut self) {
        self.clamp_to_boundaries();
        let selection = self
            .marked_range
            .clone()
            .unwrap_or_else(|| self.selection());
        let range = if selection.is_empty() {
            previous_search_boundary(&self.buffer, self.cursor)..self.cursor
        } else {
            selection
        };
        self.buffer.replace_range(range.clone(), "");
        self.cursor = range.start;
        self.anchor = self.cursor;
        self.marked_range = None;
    }

    fn delete_forward(&mut self) {
        self.clamp_to_boundaries();
        let selection = self
            .marked_range
            .clone()
            .unwrap_or_else(|| self.selection());
        let range = if selection.is_empty() {
            self.cursor..next_search_boundary(&self.buffer, self.cursor)
        } else {
            selection
        };
        self.buffer.replace_range(range.clone(), "");
        self.cursor = range.start;
        self.anchor = self.cursor;
        self.marked_range = None;
    }

    fn selected_text(&self) -> Option<&str> {
        let range = self.selection();
        (!range.is_empty()).then(|| &self.buffer[range])
    }

    fn byte_to_utf16(&self, byte: usize) -> usize {
        self.buffer[..clamp_search_boundary(&self.buffer, byte)]
            .encode_utf16()
            .count()
    }

    fn utf16_to_byte(&self, utf16: usize) -> usize {
        let mut units = 0;
        for (byte, ch) in self.buffer.char_indices() {
            if units >= utf16 {
                return byte;
            }
            let next = units + ch.len_utf16();
            if next > utf16 {
                return byte;
            }
            units = next;
        }
        self.buffer.len()
    }

    fn range_from_utf16(&self, range: Range<usize>) -> Range<usize> {
        self.utf16_to_byte(range.start)..self.utf16_to_byte(range.end)
    }
}

fn clamp_search_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn previous_search_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_search_boundary(text, offset);
    text[..offset]
        .grapheme_indices(true)
        .next_back()
        .map_or(0, |(index, _)| index)
}

fn next_search_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_search_boundary(text, offset);
    text[offset..]
        .grapheme_indices(true)
        .nth(1)
        .map_or(text.len(), |(index, _)| offset + index)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchPanelForm {
    Find,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchDomain {
    Source,
    ReadPreview,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreviewSearchMatch {
    block_index: usize,
    run_id: PreviewTextRunId,
    range: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SearchTarget {
    Source(SearchMatchRange),
    ReadPreview(PreviewSearchMatch),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SearchResultState {
    Idle,
    PendingPreview,
    InvalidPattern(String),
    NoMatches,
    Ready,
}

impl Default for SearchResultState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchGenerationKey {
    tab_index: usize,
    document_version: u64,
    domain: SearchDomain,
    query: String,
    case_sensitive: bool,
    regex: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinkEditorField {
    Label,
    Url,
    Title,
}

#[derive(Clone, Debug)]
struct LinkEditorState {
    source_range: Range<usize>,
    document_version: u64,
    label: String,
    url: String,
    title: String,
    field: LinkEditorField,
}

#[derive(Clone, Debug)]
struct RecoveryManagerState {
    entries: Vec<RecoveryInventoryEntry>,
}

#[derive(Clone, Debug)]
struct SlashCommandState {
    query: SlashQuery,
    selected: usize,
}

#[derive(Clone, Debug)]
struct BlockMenuState {
    target: BlockTarget,
    selection_format: Option<VisualSelectionFormatTarget>,
    anchor: Point<Pixels>,
    root_selected: usize,
    submenu: Option<BlockMenuSubmenu>,
    submenu_selected: usize,
}

/// An exact non-empty text selection that can safely receive one of the
/// Visual Edit inline-format commands from a contextual menu.
#[derive(Clone, Debug, PartialEq, Eq)]
struct VisualSelectionFormatTarget {
    document_version: u64,
    range: Range<usize>,
    block_id: VisualBlockId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionFormatAction {
    Bold,
    Italic,
    InlineCode,
    Link,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockMenuSubmenu {
    TextAndHeadings,
    Lists,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockMenuItem {
    SelectionFormat(SelectionFormatAction),
    Submenu(BlockMenuSubmenu),
    Transform(BlockTransform),
    Duplicate,
    MoveUp,
    MoveDown,
    Delete,
}

const BLOCK_MENU_SELECTION_FORMAT_ITEMS: [BlockMenuItem; 4] = [
    BlockMenuItem::SelectionFormat(SelectionFormatAction::Bold),
    BlockMenuItem::SelectionFormat(SelectionFormatAction::Italic),
    BlockMenuItem::SelectionFormat(SelectionFormatAction::InlineCode),
    BlockMenuItem::SelectionFormat(SelectionFormatAction::Link),
];

const BLOCK_MENU_ROOT_ITEMS: [BlockMenuItem; 10] = [
    BlockMenuItem::Submenu(BlockMenuSubmenu::TextAndHeadings),
    BlockMenuItem::Submenu(BlockMenuSubmenu::Lists),
    BlockMenuItem::Transform(BlockTransform::Quote),
    BlockMenuItem::Transform(BlockTransform::CodeBlock),
    BlockMenuItem::Transform(BlockTransform::Divider),
    BlockMenuItem::Transform(BlockTransform::Table),
    BlockMenuItem::Duplicate,
    BlockMenuItem::MoveUp,
    BlockMenuItem::MoveDown,
    BlockMenuItem::Delete,
];

const BLOCK_MENU_TEXT_ITEMS: [BlockMenuItem; 7] = [
    BlockMenuItem::Transform(BlockTransform::Text),
    BlockMenuItem::Transform(BlockTransform::Heading(1)),
    BlockMenuItem::Transform(BlockTransform::Heading(2)),
    BlockMenuItem::Transform(BlockTransform::Heading(3)),
    BlockMenuItem::Transform(BlockTransform::Heading(4)),
    BlockMenuItem::Transform(BlockTransform::Heading(5)),
    BlockMenuItem::Transform(BlockTransform::Heading(6)),
];

const BLOCK_MENU_LIST_ITEMS: [BlockMenuItem; 3] = [
    BlockMenuItem::Transform(BlockTransform::BulletedList),
    BlockMenuItem::Transform(BlockTransform::NumberedList),
    BlockMenuItem::Transform(BlockTransform::TaskList),
];

fn block_menu_root_items(has_selection_format: bool) -> Vec<BlockMenuItem> {
    let mut items = Vec::with_capacity(
        BLOCK_MENU_ROOT_ITEMS.len()
            + if has_selection_format {
                BLOCK_MENU_SELECTION_FORMAT_ITEMS.len()
            } else {
                0
            },
    );
    if has_selection_format {
        items.extend(BLOCK_MENU_SELECTION_FORMAT_ITEMS);
    }
    items.extend(BLOCK_MENU_ROOT_ITEMS);
    items
}

impl BlockMenuState {
    fn root_items(&self) -> Vec<BlockMenuItem> {
        block_menu_root_items(self.selection_format.is_some())
    }
}

impl BlockMenuSubmenu {
    fn items(self) -> &'static [BlockMenuItem] {
        match self {
            Self::TextAndHeadings => &BLOCK_MENU_TEXT_ITEMS,
            Self::Lists => &BLOCK_MENU_LIST_ITEMS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BlockMenuPresentation {
    current: BlockTransform,
    can_duplicate_or_delete: bool,
    can_move_up: bool,
    can_move_down: bool,
}

impl BlockMenuPresentation {
    fn item_enabled(self, item: BlockMenuItem) -> bool {
        match item {
            BlockMenuItem::MoveUp => self.can_move_up,
            BlockMenuItem::MoveDown => self.can_move_down,
            BlockMenuItem::Duplicate | BlockMenuItem::Delete => self.can_duplicate_or_delete,
            BlockMenuItem::SelectionFormat(_)
            | BlockMenuItem::Submenu(_)
            | BlockMenuItem::Transform(_) => true,
        }
    }
}

fn slash_command_label(language: Language, command: SlashCommand) -> String {
    match command {
        SlashCommand::Text => p1_t(language, P1Msg::TextBlock).to_string(),
        SlashCommand::Heading(level) => {
            p1_tf(language, P1Msg::Heading, &[&level.clamp(1, 6).to_string()])
        }
        SlashCommand::BulletedList => p1_t(language, P1Msg::BulletedList).to_string(),
        SlashCommand::NumberedList => p1_t(language, P1Msg::NumberedList).to_string(),
        SlashCommand::TaskList => p1_t(language, P1Msg::TaskList).to_string(),
        SlashCommand::Quote => p1_t(language, P1Msg::Quote).to_string(),
        SlashCommand::CodeBlock => p1_t(language, P1Msg::CodeBlock).to_string(),
        SlashCommand::Divider => p1_t(language, P1Msg::Divider).to_string(),
        SlashCommand::Table => p1_t(language, P1Msg::Table).to_string(),
    }
}

fn localized_slash_commands(language: Language, query: &str) -> Vec<SlashCommand> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return SlashCommand::ALL.to_vec();
    }
    SlashCommand::ALL
        .into_iter()
        .filter(|command| {
            command.search_terms().contains(&needle)
                || slash_command_label(language, *command)
                    .to_lowercase()
                    .contains(&needle)
        })
        .collect()
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
    input_selection: Rgba,
    search_match: Rgba,
    search_current: Rgba,
    invalid: Rgba,
}

fn theme_palette_from_definition(theme: &ThemeDefinition) -> ThemePalette {
    theme_palette_from_colors(theme.colors)
}

fn theme_palette_from_colors(colors: ThemeColors) -> ThemePalette {
    let active_bg = rgb(colors.active_bg);
    ThemePalette {
        app_bg: rgb(colors.app_bg),
        panel_bg: rgb(colors.panel_bg),
        surface_bg: rgb(colors.surface_bg),
        text: rgb(colors.text),
        muted: rgb(colors.muted),
        border: rgb(colors.border),
        active_bg,
        active_text: rgb(colors.active_text),
        input_selection: Rgba {
            a: 0.24,
            ..active_bg
        },
        search_match: Rgba {
            a: 0.18,
            ..active_bg
        },
        search_current: Rgba {
            a: 0.46,
            ..active_bg
        },
        invalid: rgb(0xdc2626),
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
const OUTLINE_ROW_VERTICAL_PADDING: f32 = 1.;
const OUTLINE_ROW_LINE_HEIGHT: f32 = 17.;
const OUTLINE_ROW_GAP: f32 = 0.;
const OUTLINE_DISCLOSURE_SLOT_SIZE: f32 = 16.;
const OUTLINE_DISCLOSURE_ICON_SIZE: f32 = 12.;
const READ_MODE_PREVIEW_MAX_WIDTH: f32 = 860.;
const PANE_SCROLLBAR_RESERVED_WIDTH: f32 = 15.;
const PANE_SCROLLBAR_THUMB_WIDTH: f32 = 9.;
const PANE_SCROLLBAR_MIN_THUMB_HEIGHT: f32 = 32.;
const PANE_SCROLLBAR_EDGE_INSET: f32 = 3.;
/// Nominal line height (px) of the source editor: 1.6x the default font size
/// (14px), so the derived ratio in `DocumentTypographyMetrics` stays 1.6 for
/// every size. Update together with `DEFAULT_EDITOR_FONT_SIZE`. Used both when
/// painting the editor text and by the line-based scroll helpers, so the two
/// stay in sync; the actual per-line height is measured during layout for
/// hit-testing.
const EDITOR_LINE_HEIGHT: f32 = 22.4;
/// Line height (px) of the preview pane. Independent of the editor: the preview
/// scrolls natively via its `ListState`, not by line-index math.
const PREVIEW_LINE_HEIGHT: f32 = 23.;

/// Resolved presentation metrics for one render. Preferences store only the
/// three user-facing values; every dependent metric is derived here so shaping,
/// painting, hit-testing, math rendering, and virtual-list measurement stay in
/// lockstep.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DocumentTypographyMetrics {
    pub(super) editor_font_size: f32,
    pub(super) editor_line_height: f32,
    pub(super) rendered_font_size: f32,
    pub(super) preview_row_line_height: f32,
    pub(super) paragraph_line_height: f32,
    pub(super) paragraph_spacing: f32,
    pub(super) list_line_height: f32,
    pub(super) quote_font_size: f32,
    pub(super) quote_line_height: f32,
    pub(super) source_island_font_size: f32,
    pub(super) source_island_line_height: f32,
    pub(super) code_font_size: f32,
    pub(super) code_line_height: f32,
    pub(super) small_font_size: f32,
    pub(super) table_font_size: f32,
    pub(super) inline_math_font_size: f32,
    pub(super) display_math_font_size: f32,
}

impl DocumentTypographyMetrics {
    pub(super) fn new(
        editor_font_size: u16,
        rendered_font_size: u16,
        paragraph_spacing: u16,
    ) -> Self {
        let editor_font_size = normalize_editor_font_size(editor_font_size as i64) as f32;
        let rendered_font_size = normalize_rendered_font_size(rendered_font_size as i64) as f32;
        let paragraph_spacing = normalize_paragraph_spacing(paragraph_spacing as i64) as f32;
        let rendered_scale = rendered_font_size / DEFAULT_RENDERED_FONT_SIZE as f32;
        Self {
            editor_font_size,
            editor_line_height: editor_font_size
                * (EDITOR_LINE_HEIGHT / DEFAULT_EDITOR_FONT_SIZE as f32),
            rendered_font_size,
            preview_row_line_height: PREVIEW_LINE_HEIGHT * rendered_scale,
            paragraph_line_height: 24. * rendered_scale,
            paragraph_spacing,
            list_line_height: 22. * rendered_scale,
            quote_font_size: 16. * rendered_scale,
            quote_line_height: 23. * rendered_scale,
            source_island_font_size: 13. * rendered_scale,
            source_island_line_height: 21. * rendered_scale,
            code_font_size: 12. * rendered_scale,
            code_line_height: 19. * rendered_scale,
            small_font_size: 11. * rendered_scale,
            table_font_size: 12. * rendered_scale,
            inline_math_font_size: MATH_INLINE_FONT_SIZE * rendered_scale,
            display_math_font_size: MATH_DISPLAY_FONT_SIZE * rendered_scale,
        }
    }

    pub(super) fn heading_font_size(self, level: u32) -> f32 {
        let default = match level {
            1 => 24.,
            2 => 20.,
            3 => 18.,
            _ => 16.,
        };
        default * (self.rendered_font_size / DEFAULT_RENDERED_FONT_SIZE as f32)
    }

    pub(super) fn math_font_size(self, style: MathLayoutStyle) -> f32 {
        match style {
            MathLayoutStyle::Text => self.inline_math_font_size,
            MathLayoutStyle::Display => self.display_math_font_size,
        }
    }
}

/// Resolved font family per document plane (preference over active-theme
/// contribution over built-in default). Precomputed on every change so render
/// code reads a ready value instead of re-deriving per frame.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ResolvedFontFamilies {
    pub(super) editor: SharedString,
    pub(super) rendered: SharedString,
    pub(super) code: SharedString,
}

/// One document font slot, addressed by the Preferences panel's capture input
/// and the family setter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FontSlot {
    Editor,
    Rendered,
    Code,
}

impl FontSlot {
    /// The slot's resolved family.
    pub(super) fn select(self, fonts: &ResolvedFontFamilies) -> &SharedString {
        match self {
            FontSlot::Editor => &fonts.editor,
            FontSlot::Rendered => &fonts.rendered,
            FontSlot::Code => &fonts.code,
        }
    }
}

/// Which document font slot currently has its installed-font selection list
/// open in the Preferences panel. Only one list is open at a time.
pub(super) type FontPicker = FontSlot;

/// The code-slot font with its monospace fallback chain, so code text never
/// degrades to a proportional family when the resolved primary is missing.
pub(super) fn code_slot_font(family: &SharedString) -> Font {
    Font {
        family: family.clone(),
        features: FontFeatures::default(),
        fallbacks: Some(FontFallbacks::from_fonts(vec![
            "Cascadia Code".to_string(),
            "Consolas".to_string(),
            "SFMono-Regular".to_string(),
            "Menlo".to_string(),
            "DejaVu Sans Mono".to_string(),
        ])),
        weight: FontWeight::default(),
        style: FontStyle::default(),
    }
}

impl MarkionApp {
    pub(super) fn typography_metrics(&self) -> DocumentTypographyMetrics {
        DocumentTypographyMetrics::new(
            self.editor_font_size,
            self.rendered_font_size,
            self.paragraph_spacing,
        )
    }

    /// Recomputes the resolved per-plane families from the current font
    /// preferences and the active theme. Returns the new values so callers
    /// can detect an actual change and invalidate measurements only then.
    pub(super) fn recompute_resolved_font_families(&mut self) -> ResolvedFontFamilies {
        let theme_fonts = self.active_theme_definition().fonts;
        self.resolved_font_families = ResolvedFontFamilies {
            editor: resolve_font_family(
                self.editor_font_family.as_deref(),
                theme_fonts.editor.as_deref(),
                SYSTEM_UI_FONT_FAMILY,
            )
            .into(),
            rendered: resolve_font_family(
                self.rendered_font_family.as_deref(),
                theme_fonts.rendered.as_deref(),
                SYSTEM_UI_FONT_FAMILY,
            )
            .into(),
            code: resolve_font_family(
                self.code_font_family.as_deref(),
                theme_fonts.code.as_deref(),
                DEFAULT_CODE_FONT_FAMILY,
            )
            .into(),
        };
        self.resolved_font_families.clone()
    }
}
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
const SIDEBAR_DIVIDER_WIDTH: f32 = 1.;
const DOCUMENT_TAB_BAND_HEIGHT: f32 = 30.;
/// Width bounds for one document tab in the strip. Tabs truncate their title
/// past the max and shrink toward the min as more tabs open; once every tab
/// is at the minimum the strip scrolls instead of shrinking further.
const DOCUMENT_TAB_MAX_WIDTH: f32 = 220.;
const DOCUMENT_TAB_MIN_WIDTH: f32 = 96.;

fn document_tab_band_visible(tab_count: usize) -> bool {
    tab_count > 1
}

fn document_tab_band_height(tab_count: usize) -> f32 {
    if document_tab_band_visible(tab_count) {
        DOCUMENT_TAB_BAND_HEIGHT
    } else {
        0.
    }
}

/// Drag value types used only to key `on_drag` / `on_drag_move` / `on_drop` —
/// they carry no data, they just let each divider's drag be tracked
/// independently (mirrors Zed's `DraggedSplitHandle`).
#[derive(Debug, Clone)]
struct DraggedEditorSplitHandle;
#[derive(Debug, Clone)]
struct DraggedSidebarHandle;
#[derive(Debug, Clone)]
struct DraggedVisualBlock {
    target: BlockTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneScrollTarget {
    Editor,
    Preview,
    /// Visual Edit list overlay. Drag identity only; never a Sync scroll driver.
    Visual,
    /// Preferences panel General tab body. Drag identity only.
    PreferencesGeneral,
    /// Preferences panel Appearance tab body. Drag identity only.
    PreferencesAppearance,
    /// Preferences panel Shortcuts tab category sidebar. Drag identity only.
    PreferencesShortcutCategories,
    /// Preferences panel Shortcuts tab action list. Drag identity only.
    PreferencesShortcutActions,
    /// Preferences panel Export tab body. Drag identity only.
    PreferencesExport,
    /// Files sidebar tree list. Drag identity only; never a Sync scroll driver.
    FileTree,
    /// Outline sidebar heading list. Drag identity only; never a Sync scroll driver.
    Outline,
    /// Help → Markdown Reference overlay body. Drag identity only.
    MarkdownReference,
}

/// Identity of a selectable plain-text run inside one preview list item.
/// Decorative chrome (list markers, code line numbers, table buttons) is never
/// a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewTextRunId {
    Body,
    /// Text of the n-th nested child block inside a blockquote.
    QuoteChild(usize),
    CodeBody,
    CodeLine(usize),
    MathLatex,
    HtmlText,
    TableCell {
        row: usize,
        col: usize,
    },
}

impl PreviewTextRunId {
    /// Stable document order of runs within a single preview block.
    fn rank(self) -> (u8, usize, usize) {
        match self {
            Self::Body => (0, 0, 0),
            Self::QuoteChild(i) => (0, 1 + i, 0),
            Self::CodeBody => (1, 0, 0),
            Self::CodeLine(i) => (2, i, 0),
            Self::MathLatex => (3, 0, 0),
            Self::HtmlText => (4, 0, 0),
            Self::TableCell { row, col } => (5, row, col),
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

/// Right-click menu for a workspace tab-bar item (mirrors
/// [`FileTreeContextMenu`]). Actions use switch-then-operate semantics: the
/// clicked tab becomes active before the action runs.
#[derive(Debug, Clone)]
struct TabContextMenu {
    target: TabContextTarget,
    position: Point<Pixels>,
}

/// Identity snapshot of the tab a context menu targets, captured when the
/// menu opens. The menu can stay open across tab mutations, so the stored
/// index is re-validated against this identity at dispatch time; a mismatch
/// cancels the action instead of operating on whatever tab now sits at the
/// index. `recovery_id` is globally unique per document tab state and
/// survives renames, distinguishing untitled tabs that share `path: None`.
#[derive(Debug, Clone, PartialEq)]
struct TabContextTarget {
    index: usize,
    path: Option<PathBuf>,
    recovery_id: Option<u64>,
}

impl TabContextTarget {
    fn capture(index: usize, tab: &EditorTab) -> Self {
        Self {
            index,
            path: tab.path().map(Path::to_path_buf),
            recovery_id: tab.document_tab().map(|tab| tab.recovery_id),
        }
    }

    /// True when `tab` is still the tab this target was captured from.
    fn matches(&self, tab: &EditorTab) -> bool {
        if tab.path() != self.path.as_deref() {
            return false;
        }
        match (tab.document_tab(), self.recovery_id) {
            (Some(state), Some(id)) => state.recovery_id == id,
            (None, None) => true,
            _ => false,
        }
    }

    /// Like [`Self::matches`], but follows a document across Save As (path
    /// changes, `recovery_id` does not).
    fn matches_document_identity(&self, tab: &EditorTab) -> bool {
        match (tab.document_tab(), self.recovery_id) {
            (Some(state), Some(id)) => state.recovery_id == id,
            _ => self.matches(tab),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsavedChoice {
    Save,
    Discard,
    Cancel,
}

impl UnsavedChoice {
    fn from_prompt(answer: Result<usize, impl Sized>) -> Self {
        match answer {
            Ok(0) => Self::Save,
            Ok(1) => Self::Discard,
            _ => Self::Cancel,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsavedExitKind {
    MenuQuit,
    WindowClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamedSave {
    Saved,
    Untitled,
    Conflict,
    Failed,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabContextAction {
    CloseTab,
    CloseOthers,
    CloseToTheRight,
    Rename,
    CopyPath,
    RevealInFileManager,
}

/// Menu items in display order; each inner slice becomes one group separated
/// by a hairline. Grouping lives outside the action enum so the enum stays a
/// pure dispatcher vocabulary.
const TAB_CONTEXT_ACTION_GROUPS: &[&[TabContextAction]] = &[
    &[
        TabContextAction::CloseTab,
        TabContextAction::CloseOthers,
        TabContextAction::CloseToTheRight,
    ],
    &[
        TabContextAction::Rename,
        TabContextAction::CopyPath,
        TabContextAction::RevealInFileManager,
    ],
];

/// Tabs inside the Preferences panel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PreferencesTab {
    #[default]
    General,
    Appearance,
    Shortcuts,
    Export,
}

/// A shortcut row waiting for the user to press a new key combination.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ShortcutCapture {
    action_id: String,
    error: Option<ShortcutCaptureError>,
}

/// Why a captured keystroke was rejected; rendered through the i18n layer.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ShortcutCaptureError {
    /// Bare printable key or otherwise unassignable keystroke.
    NotAssignable,
    /// Matches another action's effective binding or a reserved fixed key;
    /// payload is the display name of the conflicting target.
    Conflict(String),
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
mod export_prefs;
mod math_render;
mod memory;
mod network;
mod preview;
mod preview_image;
mod process_memory;
mod publishing;
mod root_view;
mod save_dialog;
mod search;
mod shortcuts;
mod state;
mod status_bar;
mod update;
mod workspace;

use workspace::OpenPathIntent;
#[cfg(test)]
use workspace::{ExternalDropIntent, classify_external_drop_path};

#[cfg(test)]
mod mutation_tests;
#[cfg(test)]
mod tests;

use bootstrap::{bind_app_keys, install_menus};
use diagram::*;
use editor_element::EditorElement;
use math_render::*;
use preview::*;
use preview_image::*;
use process_memory::*;
use root_view::*;
use save_dialog::*;
use shortcuts::sanitized_shortcut_overrides;
use state::*;
use status_bar::*;

pub(super) fn run() {
    bootstrap::run();
}

type HighlightCache = RefCell<HashMap<(Option<String>, String), Rc<Vec<Vec<HighlightedSpan>>>>>;

struct MarkionApp {
    tabs: Vec<EditorTab>,
    active_tab: usize,
    focus_handle: FocusHandle,
    active_menu: Option<AppMenu>,
    /// File → Open Recent nested submenu visibility. Cleared whenever
    /// `active_menu` leaves File or the whole menu closes.
    open_recent_submenu_open: bool,
    /// Whether the transient, root-hosted About Markion modal is visible.
    /// This is application chrome only and is never persisted.
    about_dialog_open: bool,
    markdown_reference_open: bool,
    /// Scroll handle for the Markdown Reference overlay body so it can show a
    /// draggable right-side scrollbar when content overflows.
    markdown_reference_scroll: ScrollHandle,
    status: SharedString,
    /// Last caption sent to the native window title. Compared before
    /// `set_window_title` so unchanged identity does not hit the platform.
    applied_window_title: Option<String>,
    /// Process-owned, lazily created loopback publishing service. `None`
    /// keeps ordinary startup/editing entirely free of bundle and socket work.
    publishing_service: Option<wechat_workspace::WorkspaceService>,
    browser_launcher: Arc<dyn publishing::BrowserLauncher>,
    /// Filesystem-derived Git context is cached separately from documents so
    /// render/input never perform repository I/O and undo snapshots stay pure.
    git_branch_state: GitBranchState,
    confirming_close: bool,
    allow_close: bool,
    preferences_path: PathBuf,
    session_path: PathBuf,
    /// Last workspace root, open saved tabs, and recent files.
    session: SessionState,
    theme: AppTheme,
    custom_theme: Option<ThemeDefinition>,
    custom_themes: Vec<ThemeDefinition>,
    themes_dir: PathBuf,
    /// Name of the active theme, used to resolve the palette across both the
    /// built-in theme table and user-loaded `.theme` files. Empty/unknown
    /// values fall back to the legacy `theme`/`custom_theme` fields.
    selected_theme_name: String,
    /// Whether the in-app Preferences panel (theme + language picker +
    /// shortcut editor) is open.
    preferences_panel_open: bool,
    /// Active tab inside the Preferences panel. Transient UI state,
    /// deliberately not persisted with editor preferences.
    preferences_tab: PreferencesTab,
    preferences_panel_focus: FocusHandle,
    /// Scroll handles for the Preferences panel's scrollable regions, so each
    /// region can show a draggable overlay scrollbar and keep its position.
    preferences_general_scroll: ScrollHandle,
    preferences_appearance_scroll: ScrollHandle,
    preferences_categories_scroll: ScrollHandle,
    preferences_actions_scroll: ScrollHandle,
    preferences_export_scroll: ScrollHandle,
    /// Cached `pandoc --version` probe result for the Export tab's
    /// availability line. `None` = not probed (or probing in flight).
    pandoc_available_cached: Option<bool>,
    shortcut_platform: ShortcutPlatform,
    shortcut_category: ShortcutCategory,
    /// Menu-action shortcut overrides loaded from `config.toml`
    /// (`[shortcuts]`, action id -> GPUI keystroke string).
    shortcut_overrides: BTreeMap<String, String>,
    /// Shortcut row currently capturing a new binding, if any. While this is
    /// set the application keymap is cleared so the captured keystroke cannot
    /// dispatch an action; ending capture rebinds everything.
    shortcut_capture: Option<ShortcutCapture>,
    focus_mode: bool,
    typewriter_mode: bool,
    code_line_numbers: bool,
    preview_adaptive_width: bool,
    editor_font_size: u16,
    rendered_font_size: u16,
    paragraph_spacing: u16,
    /// Explicit font-family preferences. `None` follows the active theme's
    /// `[fonts]` entry, then the built-in default.
    editor_font_family: Option<String>,
    rendered_font_family: Option<String>,
    code_font_family: Option<String>,
    /// Resolved per-plane families (preference over theme over default),
    /// recomputed whenever a font preference or the active theme changes so
    /// render code never re-derives per frame.
    resolved_font_families: ResolvedFontFamilies,
    /// Font slot whose installed-font selection list is open in the
    /// Preferences panel, if any.
    font_picker: Option<FontPicker>,
    /// Installed font family names, refreshed when the Preferences panel
    /// opens; powers the panel's advisory not-installed warning.
    installed_font_names: Vec<String>,
    heading_menu_max_level: u8,
    /// When enabled and the view mode is Split, the editor and preview panes
    /// follow the same source-backed document location. Persisted; disabled by
    /// default.
    sync_scroll: bool,
    /// When enabled, the file-tree panel lists hidden entries (dotfile names,
    /// plus Windows-hidden-attribute entries on Windows). Persisted; disabled
    /// by default. The always-excluded noise list stays excluded regardless.
    show_hidden_files: bool,
    /// When enabled, non-explicit opens (File → Open, file-tree click,
    /// drag-drop, Open Recent) replace a safe-to-replace active tab instead
    /// of appending a new tab. Persisted; enabled by default.
    open_in_current_tab: bool,
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
    outline_scroll: ScrollHandle,
    /// Horizontal scroll position of the document tab strip. Window-
    /// presentation state: deliberately not per-tab, so switching tabs never
    /// restores a foreign strip offset.
    tab_bar_scroll: ScrollHandle,
    /// Test seam: the tab index most recently requested by
    /// `reveal_active_tab_in_strip`. GPUI's pending-reveal state is private,
    /// so behavioral tests assert against this instead.
    #[cfg(test)]
    last_tab_strip_reveal: Option<usize>,
    // Byte length of the trailing IME composition inside whichever redirected
    // text field (file-tree filter / search) currently has logical focus.
    input_marked_len: usize,
    /// Last document identity/version reported to the platform text service.
    /// A callback for another tab or an older version is rejected instead of
    /// being reinterpreted against the current active document.
    document_input_target: Option<(DocumentInstanceId, u64)>,
    /// Version chain owned by the active IME composition.
    ime_input_target: Option<(DocumentInstanceId, u64)>,
    selected_tree_path: Option<PathBuf>,
    collapsed_tree_paths: HashSet<PathBuf>,
    /// Set when a replacement workspace root still needs its first successful
    /// scan to seed the one-level default tree view.
    file_tree_needs_initial_collapse: bool,
    file_tree_context_menu: Option<FileTreeContextMenu>,
    /// Right-click menu for the rendered preview pane.
    preview_context_menu: Option<PreviewContextMenu>,
    /// Right-click menu for a tab-bar item.
    tab_context_menu: Option<TabContextMenu>,
    /// Open inline name editor for a file-tree create/rename action; reuses
    /// the redirected-text-input path so keystrokes route into its buffer.
    pending_name_input: Option<PendingNameInput>,
    /// True between the mouse-down that click-away-committed the inline name
    /// editor and the mouse-up that consumes the same click. Tree-row and
    /// tab-strip mouse-up handlers check it so the click that committed a
    /// rename does not also open a file or switch tabs.
    name_editor_click_away: bool,
    /// Image bytes waiting for a durable Markdown base path. Set when an image
    /// is pasted/dropped into an untitled tab and consumed after Save As.
    pending_image_import: Option<Vec<PendingImageInput>>,
    link_editor: Option<LinkEditorState>,
    /// Startup crash snapshots awaiting an explicit per-entry decision. This
    /// remains open independently of the active editor tab so unreadable or
    /// deferred snapshots are never silently removed.
    recovery_manager: Option<RecoveryManagerState>,
    slash_commands: Option<SlashCommandState>,
    dismissed_slash_query: Option<SlashQuery>,
    block_menu: Option<BlockMenuState>,
    search_visible: bool,
    /// Whether replacement controls are currently available. The requested
    /// form is retained separately so Read mode can temporarily present Find.
    replace_visible: bool,
    search_form: SearchPanelForm,
    search_query: SearchFieldState,
    replace_text: SearchFieldState,
    search_case_sensitive: bool,
    search_regex: bool,
    search_focus: Option<SearchField>,
    search_control_focus: Option<SearchOverlayControl>,
    search_matches: Vec<SearchTarget>,
    current_search_index: Option<usize>,
    search_result: SearchResultState,
    search_generation: Option<SearchGenerationKey>,
    search_field_bounds: [Option<Bounds<Pixels>>; 2],
    pane_scrollbar_drag: Option<PaneScrollbarDrag>,
    /// Auto-save settings from `[auto_save]`. `silent_save` and `delay_secs`
    /// are editable in Preferences → General; `enabled` remains file-only.
    auto_save_preferences: AutoSavePreferences,
    /// Export settings from the config file ([export] table). Not editable
    /// in the Preferences panel; kept to round-trip on save.
    export_preferences: ExportPreferences,
    recovery_dir: PathBuf,
    /// One background external-change round at a time: while the disk work of
    /// `check_external_changes` is in flight, further poll ticks are skipped
    /// so a stalled filesystem cannot pile up blocked background tasks.
    external_check_in_flight: bool,
    /// Memoized heavy-slot classification for preview-image decode
    /// scheduling. The probe opens the image header on disk, so it runs on
    /// the background executor and the result is cached here; keys are image
    /// identities, so a stale entry can at worst misclassify a slot, never
    /// corrupt a decode.
    preview_probe_results: HashMap<PreviewImageKey, bool>,
    preview_probes_in_flight: HashSet<PreviewImageKey>,
    /// Memoized syntax highlighting keyed by (language, code). Preview blocks
    /// are re-collected on every edit, but the code blocks themselves rarely
    /// change while typing prose, so their token spans are reused across
    /// edits instead of being re-lexed on every keystroke.
    highlight_cache: HighlightCache,
    /// Shared across tabs and frames; pending entries are never evicted.
    diagram_cache: DiagramCache,
    /// Decoded Markdown preview images owned by Markion (not GPUI loading_assets).
    preview_image_cache: PreviewImageCache,
    /// Presentation-only formula results shared across tabs and document versions.
    math_cache: MathCache,
    /// Active interface language. Persisted via `AppPreferences::language`.
    language: Language,
    /// Whether `MarkionApp::new` should schedule a silent update check on
    /// startup. Persisted via `AppPreferences::check_for_updates_on_startup`.
    check_for_updates_on_startup: bool,
    /// ISO-8601 timestamp of the most recent update check. Persisted via
    /// `AppPreferences::last_update_check`; used to throttle startup checks.
    last_update_check: Option<String>,
}
