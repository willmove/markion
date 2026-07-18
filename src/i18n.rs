//! Hand-rolled internationalization (i18n) for Markion.
//!
//! Matches the project's minimal-dependency style: no external crate, just an
//! enum of compile-time-checked message keys, a [`Language`] selector, and two
//! lookup functions ([`t`] for static labels, [`tf`] for templates with
//! positional `{0}`/`{1}`/... arguments). Adding a new language is a matter of
//! extending the `match` arms; missing a translation site is a compile error.
//!
//! Only user-visible UI chrome is translated. Document content (the welcome
//! Markdown, user files) is left untouched.

use crate::model::SidebarTab;

/// Supported interface languages. Add a variant here and fill in the `match`
/// arms in [`t`] / [`tf`] / [`shortcut_catalog`] to ship a new language.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    En,
    /// Simplified Chinese (简体中文). Persisted as `zh-hans`; also accepts the
    /// legacy aliases `zh` / `chs` / `zh-cn` / `chinese` for backwards compat.
    ZhHans,
    /// Traditional Chinese (繁體中文). Persisted as `zh-hant`.
    ZhHant,
    Ja,
    Fr,
    De,
    Es,
}

impl Language {
    /// Stable, lowercase persistence code written to the preferences file.
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::ZhHans => "zh-hans",
            Self::ZhHant => "zh-hant",
            Self::Ja => "ja",
            Self::Fr => "fr",
            Self::De => "de",
            Self::Es => "es",
        }
    }

    /// Parse a preference value back into a [`Language`]. Unknown / empty
    /// values fall back to [`Language::default`] (English), mirroring how
    /// `sidebar_tab` and other prefs tolerate forward-compat values.
    pub fn from_code(code: &str) -> Self {
        match code.trim().to_ascii_lowercase().as_str() {
            "zh" | "chs" | "zh-cn" | "zh-hans" | "chinese" => Self::ZhHans,
            "zh-hant" | "zh-tw" | "zh-hk" | "cht" | "traditional chinese" => Self::ZhHant,
            "ja" | "jp" | "japanese" | "jpn" => Self::Ja,
            "fr" | "francais" | "français" | "french" | "fra" => Self::Fr,
            "de" | "deutsch" | "german" | "ger" | "deu" => Self::De,
            "es" | "espanol" | "español" | "spanish" | "spa" => Self::Es,
            // English is the safe default for anything unrecognised.
            _ => Self::En,
        }
    }

    /// Native display name used in the View → Language submenu (the name is
    /// always shown in its own script, following common OS conventions).
    pub fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::ZhHans => "简体中文",
            Self::ZhHant => "繁體中文",
            Self::Ja => "日本語",
            Self::Fr => "Français",
            Self::De => "Deutsch",
            Self::Es => "Español",
        }
    }

    /// All selectable languages, in display order. The two Chinese variants
    /// sit next to each other so users can pick their script at a glance.
    pub fn all() -> &'static [Language] {
        &[
            Self::En,
            Self::ZhHans,
            Self::ZhHant,
            Self::De,
            Self::Es,
            Self::Fr,
            Self::Ja,
        ]
    }
}

/// Compile-time-checked UI message keys. One variant per distinct user-visible
/// string. Dynamic bits (paths, counts) are interpolated via [`tf`] using
/// `{0}` / `{1}` placeholders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    // --- Menu bar titles ---
    MenuFile,
    MenuEdit,
    MenuView,
    MenuFormat,
    MenuExport,
    MenuHelp,

    // --- File menu items ---
    ItemNew,
    ItemOpen,
    ItemOpenFolder,
    ItemSave,
    ItemSaveAs,
    /// Open a fresh empty document in a new tab.
    ItemNewTab,
    /// Open a file chosen from a dialog in a new tab.
    ItemOpenInNewTab,
    ItemCloseTab,
    ItemNextTab,
    ItemPrevTab,
    ItemExit,

    // --- Edit menu items ---
    ItemUndo,
    ItemRedo,
    ItemCopy,
    ItemCut,
    ItemPaste,
    ItemSelectAll,

    // Preview context menu
    ItemPreviewCopyPlain,
    ItemPreviewCopyMarkdown,
    ItemPreviewCopyHtml,
    ItemPreviewSelectAll,
    ItemPreviewCopyLinkAddress,

    // --- View menu items ---
    ItemToggleView,
    ItemEditMode,
    ItemVisualEditMode,
    ItemSplitPreviewMode,
    ItemReadMode,
    ItemToggleSidebar,
    ItemFiles,
    ItemOutline,
    ItemFocusMode,
    ItemTypewriterMode,
    ItemCodeLineNumbers,
    ItemFind,
    ItemReplace,
    ItemFindNext,
    ItemFindPrevious,
    /// "Cycle theme" action in the View menu.
    ItemCycleTheme,
    /// "Language" submenu header inside View.
    ItemLanguage,

    // --- Format menu items ---
    ItemBold,
    ItemItalic,
    ItemInlineCode,
    ItemLink,
    ItemImage,
    ItemH1,
    ItemH2,
    ItemH3,
    ItemH4,
    ItemH5,
    ItemH6,
    ItemBullets,
    ItemNumbers,
    ItemTask,
    ItemQuote,
    ItemCodeFence,
    ItemFormatTable,
    ItemAddTableRow,
    ItemDeleteTableRow,
    ItemMoveRowUp,
    ItemMoveRowDown,
    ItemAddTableColumn,
    ItemDeleteTableColumn,

    // --- Export menu items ---
    ItemExportHtml,
    ItemExportPlainHtml,
    ItemExportPdf,
    ItemExportLatex,
    ItemExportDocx,
    ItemExportPng,
    ItemExportJpeg,

    // --- Help menu items ---
    ItemPreferences,
    ItemResetPreferences,
    ItemKeyboardShortcuts,
    ItemAboutMarkion,

    // --- Panel / tab labels ---
    LabelEditor,
    LabelPreview,
    /// Generic "Files" sidebar tab header.
    LabelFiles,
    /// Generic "Outline" sidebar tab header.
    LabelOutline,
    /// "Table" preview block toolbar heading.
    LabelTable,
    LabelImageAlt,
    LabelImageDestination,
    LabelImageTitle,

    // --- Search panel ---
    SearchFind,
    SearchReplace,
    SearchPrev,
    SearchNext,
    SearchAll,
    /// Literal (non-regex) search toggle label.
    SearchLiteral,
    /// Regex search toggle label, shown as ".*".
    SearchRegexMark,
    /// Case-insensitive toggle label, shown as "aa".
    SearchCaseInsensitiveMark,
    /// Case-sensitive toggle label, shown as "Aa".
    SearchCaseSensitiveMark,
    /// "{current}/{total}" progress summary, e.g. "2/5".
    SearchProgress,

    // --- Status bar: static ---
    StatusReady,
    StatusRecoveryAvailable,
    StatusRecoveredDocument,
    StatusRecoveryDiscarded,
    StatusWaitingFileTreeConfirm,
    StatusOpenCanceled,
    StatusOpenFolderCanceled,
    StatusJumpedToHeading,
    StatusNewDocument,
    StatusOpening,
    StatusOpeningFolder,
    StatusChoosingSaveLocation,
    StatusEditMode,
    StatusVisualEditMode,
    StatusSplitPreviewMode,
    StatusReadMode,
    StatusSidebarShown,
    StatusSidebarHidden,
    StatusOutlineShown,
    StatusFileTreeShown,
    StatusFilteringFiles,
    StatusFileTreeRefreshed,
    StatusSelectTreeEntryFirst,
    StatusSaveBeforeRename,
    StatusWaitingDeleteConfirm,
    StatusDeleteCanceled,
    StatusAboutMarkion,
    StatusKeyboardShortcuts,
    StatusUndo,
    StatusNothingToUndo,
    StatusRedo,
    StatusNothingToRedo,
    StatusNoFormattingChange,
    StatusNoTableAtCursor,
    StatusTableAlreadyFormatted,
    StatusWaitingConfirm,
    StatusCanceled,
    StatusExitingMarkion,
    StatusWaitingExitConfirm,
    StatusExitCanceled,
    StatusWaitingQuitConfirm,
    StatusEditingFindQuery,
    StatusEditingReplacement,
    StatusEditing,
    StatusIndentedSelection,
    StatusOutdentedSelection,
    StatusNothingToIndent,
    StatusNothingToOutdent,
    StatusNoEdit,
    StatusClipboardEmpty,
    StatusCopiedSelection,
    StatusCopiedPreviewPlain,
    StatusCopiedPreviewMarkdown,
    StatusCopiedPreviewHtml,
    StatusCopiedLinkAddress,
    StatusPreviewSelectedAll,
    StatusNothingToCopy,
    StatusCutSelection,
    StatusNothingToCut,
    StatusComposing,
    StatusSelectedTreeEntry,
    StatusNoMatchSelected,
    StatusReplacedCurrent,
    StatusNoMatchesToReplace,
    StatusFindQueryEmpty,
    StatusNoMatches,

    // --- Format / table edit success status ---
    StatusFmtBold,
    StatusFmtItalic,
    StatusFmtInlineCode,
    StatusFmtLink,
    StatusFmtImage,
    /// {0}=level number, e.g. "Heading 1".
    StatusFmtHeading,
    StatusFmtBulletedList,
    StatusFmtNumberedList,
    StatusFmtTaskList,
    StatusFmtBlockQuote,
    StatusFmtCodeBlock,
    StatusFmtFormatTable,
    StatusFmtAddRow,
    StatusFmtDeleteRow,
    StatusFmtMoveRowUp,
    StatusFmtMoveRowDown,
    StatusFmtAddColumn,
    StatusFmtDeleteColumn,
    StatusFocusModeOn,
    StatusFocusModeOff,
    StatusTypewriterModeOn,
    StatusTypewriterModeOff,
    StatusCodeLineNumbersOn,
    StatusCodeLineNumbersOff,
    StatusRegexFind,
    StatusLiteralFind,
    StatusPreferences,
    StatusWaitingPreferenceResetConfirm,
    StatusPreferencesReset,
    StatusPreviewAdaptiveWidthOn,
    StatusPreviewAdaptiveWidthOff,
    /// Status: Sync scroll enabled.
    StatusSyncScrollOn,
    /// Status: Sync scroll disabled.
    StatusSyncScrollOff,
    StatusPreferenceResetCanceled,
    StatusLanguageSet,

    // --- Status bar: dynamic (use tf) ---
    /// {0}=err — "Recovery failed: {err}"
    StatusRecoveryFailed,
    /// {0}=path — "Opened {path}"
    StatusOpened,
    /// {0}=err — "Open failed: {err}"
    StatusOpenFailed,
    /// {0}=path — "Opened folder {path}"
    StatusOpenedFolder,
    /// {0}=err — "Open folder failed: {err}"
    StatusOpenFolderFailed,
    /// {0}=path — "Auto-saved {path}"
    StatusAutoSaved,
    /// {0}=path — "Recovery saved {path}"
    StatusRecoverySaved,
    /// {0}=err — "Auto-save failed: {err}"
    StatusAutoSaveFailed,
    /// {0}=err — "Find failed: {err}"
    StatusFindFailed,
    /// {0}=err — "Preferences save failed: {err}"
    StatusPreferencesSaveFailed,
    /// {0}=err — "Sample theme save failed: {err}"
    StatusSampleThemeSaveFailed,
    /// {0}=path — "Saved {path}"
    StatusSaved,
    /// {0}=err — "Save failed: {err}"
    StatusSaveFailed,
    /// "Save canceled"
    StatusSaveCanceled,
    /// "Export canceled"
    StatusExportCanceled,
    /// {0}=ext — "Choosing .{ext} export location..."
    StatusChoosingExportLocation,
    /// {0}=path — "Exported {path}"
    StatusExported,
    /// {0}=path — PDF/DOCX produced by the pandoc engine
    StatusExportedEngine,
    /// {0}=path — PDF/DOCX produced by the built-in writer (pandoc hint)
    StatusExportedBuiltin,
    /// {0}=err — "Export failed: {err}"
    StatusExportFailed,
    /// {0}=path — "Created {path}"
    StatusCreated,
    /// {0}=err — "Create file failed: {err}"
    StatusCreateFileFailed,
    /// {0}=err — "Create folder failed: {err}"
    StatusCreateFolderFailed,
    /// {0}=path — "Renamed to {path}"
    StatusRenamedTo,
    /// {0}=err — "Rename failed: {err}"
    StatusRenameFailed,
    /// {0}=path — "Deleted {path}"
    StatusDeleted,
    /// {0}=err — "Delete failed: {err}"
    StatusDeleteFailed,
    /// {0}=path — "Shown in system file manager: {path}"
    StatusShownInFileManager,
    /// {0}=err — "Show in system file manager failed: {err}"
    StatusShowInFileManagerFailed,
    /// {0}=theme label — "Theme: {theme}"
    StatusTheme,
    /// {0}=count — "{count} matches"
    StatusMatches,
    /// {0}=err — "Replace failed: {err}"
    StatusReplaceFailed,
    /// {0}=count — "Replaced {count} matches"
    StatusReplacedMatches,
    /// {0}=index {1}=total {2}=line {3}=col — "Match {i} of {n} at {line}:{col}"
    StatusMatchPosition,
    /// {0}=count — "{count} files visible"
    StatusFilesVisible,
    /// {0}=count — "{count} file matches"
    StatusFileMatches,

    // --- Dialogs ---
    DialogButtonOk,
    DialogButtonCancel,
    DialogButtonDiscard,
    DialogButtonDelete,
    DialogButtonReset,
    DialogButtonRestore,
    DialogButtonExitWithoutSaving,

    /// About Markion dialog title.
    DialogAboutTitle,
    /// About detail body. {0}=version {1}=repo url.
    DialogAboutDetail,
    /// Keyboard Shortcuts dialog title.
    DialogShortcutsTitle,
    /// Preferences dialog title.
    DialogPreferencesTitle,
    /// Preferences summary body. {0..8}=values, see preferences_detail().
    DialogPreferencesDetail,
    /// Restore-unsaved-document prompt title.
    DialogRestoreTitle,
    /// Restore-unsaved-document detail. {0}=path.
    DialogRestoreDetail,
    /// Discard-unsaved-changes prompt title (new / open / open-from-tree).
    DialogDiscardTitle,
    /// Discard detail for "new document". (static)
    DialogDiscardNewDetail,
    /// Discard detail for "open another document". (static)
    DialogDiscardOpenDetail,
    /// Discard detail for opening a tree file. {0}=path.
    DialogDiscardOpenTreeDetail,
    /// Exit-without-saving prompt title.
    DialogExitTitle,
    /// Exit-without-saving detail. (static)
    DialogExitDetail,
    /// Delete-file prompt title.
    DialogDeleteTitle,
    /// Delete-file detail. {0}=path.
    DialogDeleteDetail,
    /// Recursive folder delete prompt title (shown as a second confirmation
    /// when the target is a non-empty folder).
    DialogDeleteFolderRecursiveTitle,
    /// Recursive folder delete detail. {0}=folder path.
    DialogDeleteFolderRecursiveDetail,
    /// Reset-preferences prompt title.
    DialogResetTitle,
    /// Reset-preferences detail. (static)
    DialogResetDetail,
    /// Path prompt labels (open / save / export).
    PromptOpenMarkdown,
    PromptOpenFolder,
    FileTypeMarkdown,
    FileTypeStyledHtml,
    FileTypePlainHtml,
    FileTypePdf,
    FileTypeLatex,
    FileTypeDocx,
    FileTypePng,
    FileTypeJpeg,

    // --- File / outline summary strings shared by sidebar ---
    /// Word used for the "files" sidebar-tab summary token.
    SummaryFilesUnit,
    SummaryMatchesUnit,

    // --- File tree panel ---
    /// File-tree filter input placeholder (empty query).
    FileTreeFilterPlaceholder,
    /// File-tree filter input with an active query. {0}=query.
    FileTreeFilterActive,
    /// Root label fallback when the workspace name is unavailable.
    FileTreeWorkspaceFallback,
    /// "{count} more entries hidden" hint when the visible list is capped.
    /// {0}=hidden count.
    FileTreeMoreHidden,
    /// Empty-state placeholder shown in the Files panel while no file is open
    /// (the welcome document) and the tree has not been scanned yet.
    FileTreeEmptyState,
    FileTreeContextOpen,
    FileTreeContextOpenInNewTab,
    FileTreeContextCreateFile,
    FileTreeContextCreateFolder,
    FileTreeContextRename,
    FileTreeContextDelete,
    FileTreeContextShowInFileManager,
    FileTreeContextRefresh,
    FileTreeContextFilterFiles,
    /// Inline name-prompt label prefix shown as "Name: <buffer>".
    FileTreeNamePromptLabel,
    /// Status hint while the name prompt is open and being edited.
    StatusNamingEntry,
    /// Status warning when the user confirms an empty name (no entry created).
    StatusNameRequired,

    // --- Preferences summary tokens ---
    /// "on" indicator in the preferences summary.
    PrefOn,
    /// "off" indicator in the preferences summary.
    PrefOff,
    /// "hidden" indicator for the sidebar row.
    PrefSidebarHidden,
    /// "{name} (custom)" theme label. {0}=theme name.
    CustomThemeLabel,

    // --- Preferences panel ---
    /// Preferences panel title ("Preferences").
    PrefPanelTitle,
    /// Section header for the theme grid.
    PrefPanelThemeSection,
    /// Section header for the language row.
    PrefPanelLanguageSection,
    /// Section header for the other-settings summary.
    PrefPanelOtherSection,
    /// "Focus mode" row label in the other-settings summary.
    PrefPanelFocusMode,
    /// "Typewriter mode" row label.
    PrefPanelTypewriterMode,
    /// "Code line numbers" row label.
    PrefPanelCodeLineNumbers,
    /// "Preview adaptive width" row label.
    PrefPanelPreviewAdaptiveWidth,
    /// "Sync scroll" row label.
    PrefPanelSyncScroll,
    /// "Sidebar" row label.
    PrefPanelSidebar,
    /// "Heading menu" row label.
    PrefPanelHeadingMenu,
    /// H1–H5 heading depth option.
    PrefPanelHeadingMenuThree,
    /// H1–H6 heading depth option.
    PrefPanelHeadingMenuSix,
    /// Close (×) button tooltip/aria-style label for the panel.
    PrefPanelClose,

    // --- Diagram preview ---
    DiagramLoading,
    DiagramUnsupported,
    DiagramInputTooLarge,
    DiagramInvalidSource,
    DiagramUnsafeOutput,
    DiagramRenderFailed,

    // --- Math preview ---
    MathRendering,
    MathInvalid,
    MathTooLarge,
    MathUnsupported,
    MathRenderFailed,

    /// "Modified" save-state token in the title bar.
    TitleModified,
    /// "Saved" save-state token in the title bar.
    TitleSaved,
}

/// Look up a static (non-parameterised) message for `lang`.
pub fn t(lang: Language, msg: Msg) -> &'static str {
    match lang {
        Language::En => en(msg),
        Language::ZhHans => zh(msg),
        Language::ZhHant => zh_hant(msg),
        Language::Ja => ja(msg),
        Language::Fr => fr(msg),
        Language::De => de(msg),
        Language::Es => es(msg),
    }
}

/// Look up a templated message and substitute positional `{0}`/`{1}`/...
/// placeholders with `args`. Unknown placeholders are left verbatim.
pub fn tf(lang: Language, msg: Msg, args: &[&str]) -> String {
    let template: &str = t(lang, msg);
    substitute(template, args)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutPlatform {
    WindowsLinux,
    MacOS,
}

impl ShortcutPlatform {
    pub const ALL: [Self; 2] = [Self::WindowsLinux, Self::MacOS];

    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOS
        } else {
            Self::WindowsLinux
        }
    }

    pub const fn label(self, _lang: Language) -> &'static str {
        match self {
            Self::WindowsLinux => "Windows/Linux",
            Self::MacOS => "macOS",
        }
    }
}

impl Default for ShortcutPlatform {
    fn default() -> Self {
        Self::current()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShortcutCategory {
    #[default]
    Files,
    Tabs,
    Editing,
    View,
    Search,
    Tables,
    Export,
}

impl ShortcutCategory {
    pub const ALL: [Self; 7] = [
        Self::Files,
        Self::Tabs,
        Self::Editing,
        Self::View,
        Self::Search,
        Self::Tables,
        Self::Export,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShortcutKeys {
    windows_linux: &'static [&'static str],
    macos: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortcutAction {
    pub label: &'static str,
    windows_linux: &'static [&'static str],
    macos: &'static [&'static str],
}

impl ShortcutAction {
    pub const fn combinations(self, platform: ShortcutPlatform) -> &'static [&'static str] {
        match platform {
            ShortcutPlatform::WindowsLinux => self.windows_linux,
            ShortcutPlatform::MacOS => self.macos,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutSection {
    pub category: ShortcutCategory,
    pub label: &'static str,
    pub actions: Vec<ShortcutAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutCatalog {
    pub sections: Vec<ShortcutSection>,
}

impl ShortcutCatalog {
    pub fn section(&self, category: ShortcutCategory) -> Option<&ShortcutSection> {
        self.sections
            .iter()
            .find(|section| section.category == category)
    }
}

pub fn shortcut_catalog(lang: Language, heading_menu_max_level: u8) -> ShortcutCatalog {
    let extended = heading_menu_max_level >= crate::model::EXTENDED_HEADING_MENU_MAX_LEVEL;
    build_shortcut_catalog(shortcut_labels(lang), extended)
}

struct ShortcutLabels {
    sections: [&'static str; 7],
    files: [&'static str; 5],
    tabs: [&'static str; 3],
    editing: [&'static str; 7],
    view: [&'static str; 14],
    search: [&'static str; 3],
    tables: [&'static str; 4],
    export: [&'static str; 7],
}

const FILE_KEYS: [ShortcutKeys; 5] = [
    ShortcutKeys {
        windows_linux: &["Ctrl+N"],
        macos: &["Cmd+N"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+O"],
        macos: &["Cmd+O"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+S"],
        macos: &["Cmd+S"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+S"],
        macos: &["Cmd+Shift+S"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Q"],
        macos: &["Cmd+Q"],
    },
];

const TAB_KEYS: [ShortcutKeys; 3] = [
    ShortcutKeys {
        windows_linux: &["Ctrl+T"],
        macos: &["Cmd+T"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+W"],
        macos: &["Cmd+W"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Tab", "Ctrl+Shift+Tab"],
        macos: &["Ctrl+Tab", "Ctrl+Shift+Tab"],
    },
];

const EDITING_KEYS: [ShortcutKeys; 7] = [
    ShortcutKeys {
        windows_linux: &["Ctrl+B"],
        macos: &["Cmd+B"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+I"],
        macos: &["Cmd+I"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+E"],
        macos: &["Cmd+E"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+K"],
        macos: &["Cmd+K"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+I"],
        macos: &["Cmd+Shift+I"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+1", "Ctrl+2", "Ctrl+3", "Ctrl+4", "Ctrl+5"],
        macos: &["Cmd+1", "Cmd+2", "Cmd+3", "Cmd+4", "Cmd+5"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Z", "Ctrl+Y"],
        macos: &["Cmd+Z", "Cmd+Y"],
    },
];

const EDITING_KEYS_EXTENDED: [ShortcutKeys; 7] = [
    ShortcutKeys {
        windows_linux: &["Ctrl+B"],
        macos: &["Cmd+B"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+I"],
        macos: &["Cmd+I"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+E"],
        macos: &["Cmd+E"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+K"],
        macos: &["Cmd+K"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+I"],
        macos: &["Cmd+Shift+I"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+1", "Ctrl+2", "Ctrl+3", "Ctrl+4", "Ctrl+5", "Ctrl+6"],
        macos: &["Cmd+1", "Cmd+2", "Cmd+3", "Cmd+4", "Cmd+5", "Cmd+6"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Z", "Ctrl+Y"],
        macos: &["Cmd+Z", "Cmd+Y"],
    },
];

const VIEW_KEYS: [ShortcutKeys; 14] = [
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+V"],
        macos: &["Cmd+Shift+V"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+1"],
        macos: &["Cmd+Option+1"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+4"],
        macos: &["Cmd+Option+4"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+2"],
        macos: &["Cmd+Option+2"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+3"],
        macos: &["Cmd+Option+3"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+B"],
        macos: &["Cmd+Shift+B"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+F"],
        macos: &["Cmd+Shift+F"],
    },
    ShortcutKeys {
        windows_linux: &["F6"],
        macos: &["F6"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+F"],
        macos: &["Cmd+Option+F"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+T"],
        macos: &["Cmd+Shift+T"],
    },
    ShortcutKeys {
        windows_linux: &["F7"],
        macos: &["F7"],
    },
    ShortcutKeys {
        windows_linux: &["F8"],
        macos: &["F8"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+4"],
        macos: &["Cmd+Shift+4"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+,"],
        macos: &["Cmd+,"],
    },
];

const SEARCH_KEYS: [ShortcutKeys; 3] = [
    ShortcutKeys {
        windows_linux: &["Ctrl+F"],
        macos: &["Cmd+F"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+H"],
        macos: &["Cmd+H"],
    },
    ShortcutKeys {
        windows_linux: &["F3", "Shift+F3"],
        macos: &["F3", "Shift+F3"],
    },
];

const TABLE_KEYS: [ShortcutKeys; 4] = [
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+M"],
        macos: &["Cmd+Shift+M"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+Enter", "Ctrl+Alt+Backspace"],
        macos: &["Cmd+Option+Enter", "Cmd+Option+Backspace"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+Up", "Ctrl+Alt+Down"],
        macos: &["Cmd+Option+Up", "Cmd+Option+Down"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+Right", "Ctrl+Alt+Left"],
        macos: &["Cmd+Option+Right", "Cmd+Option+Left"],
    },
];

const EXPORT_KEYS: [ShortcutKeys; 7] = [
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+H"],
        macos: &["Cmd+Shift+H"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+Shift+H"],
        macos: &["Cmd+Option+Shift+H"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+P"],
        macos: &["Cmd+Shift+P"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+L"],
        macos: &["Cmd+Shift+L"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+D"],
        macos: &["Cmd+Shift+D"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Shift+G"],
        macos: &["Cmd+Shift+G"],
    },
    ShortcutKeys {
        windows_linux: &["Ctrl+Alt+Shift+G"],
        macos: &["Cmd+Option+Shift+G"],
    },
];

fn build_shortcut_catalog(labels: ShortcutLabels, extended: bool) -> ShortcutCatalog {
    ShortcutCatalog {
        sections: vec![
            build_shortcut_section(
                ShortcutCategory::Files,
                labels.sections[0],
                &labels.files,
                &FILE_KEYS,
            ),
            build_shortcut_section(
                ShortcutCategory::Tabs,
                labels.sections[1],
                &labels.tabs,
                &TAB_KEYS,
            ),
            build_shortcut_section(
                ShortcutCategory::Editing,
                labels.sections[2],
                &labels.editing,
                if extended {
                    &EDITING_KEYS_EXTENDED
                } else {
                    &EDITING_KEYS
                },
            ),
            build_shortcut_section(
                ShortcutCategory::View,
                labels.sections[3],
                &labels.view,
                &VIEW_KEYS,
            ),
            build_shortcut_section(
                ShortcutCategory::Search,
                labels.sections[4],
                &labels.search,
                &SEARCH_KEYS,
            ),
            build_shortcut_section(
                ShortcutCategory::Tables,
                labels.sections[5],
                &labels.tables,
                &TABLE_KEYS,
            ),
            build_shortcut_section(
                ShortcutCategory::Export,
                labels.sections[6],
                &labels.export,
                &EXPORT_KEYS,
            ),
        ],
    }
}

fn build_shortcut_section(
    category: ShortcutCategory,
    title: &'static str,
    actions: &[&'static str],
    keys: &[ShortcutKeys],
) -> ShortcutSection {
    debug_assert_eq!(actions.len(), keys.len());
    ShortcutSection {
        category,
        label: title,
        actions: actions
            .iter()
            .zip(keys.iter())
            .map(|(&label, keys)| ShortcutAction {
                label,
                windows_linux: keys.windows_linux,
                macos: keys.macos,
            })
            .collect(),
    }
}

fn shortcut_labels(lang: Language) -> ShortcutLabels {
    match lang {
        Language::En => ShortcutLabels {
            sections: [
                "Files", "Tabs", "Editing", "View", "Search", "Tables", "Export",
            ],
            files: ["New", "Open", "Save", "Save As", "Exit"],
            tabs: ["Open in New Tab", "Close Tab", "Next/Previous Tab"],
            editing: [
                "Bold",
                "Italic",
                "Inline Code",
                "Link",
                "Image",
                "Headings",
                "Undo/Redo",
            ],
            view: [
                "Cycle View Mode",
                "Edit Mode",
                "Visual Edit Mode",
                "Split Preview Mode",
                "Read Mode",
                "Sidebar",
                "Files",
                "Outline",
                "Filter Files",
                "Theme",
                "Focus Mode",
                "Typewriter Mode",
                "Code Line Numbers",
                "Preferences",
            ],
            search: ["Find", "Replace", "Next/Previous Match"],
            tables: [
                "Format Table",
                "Add/Delete Row",
                "Move Row",
                "Add/Delete Column",
            ],
            export: ["HTML", "Plain HTML", "PDF", "LaTeX", "DOCX", "PNG", "JPEG"],
        },
        Language::ZhHans => ShortcutLabels {
            sections: ["文件", "标签页", "编辑", "视图", "搜索", "表格", "导出"],
            files: ["新建", "打开", "保存", "另存为", "退出"],
            tabs: ["在新标签页中打开", "关闭标签页", "下一个/上一个标签页"],
            editing: [
                "粗体",
                "斜体",
                "行内代码",
                "链接",
                "图片",
                "标题",
                "撤销/重做",
            ],
            view: [
                "切换视图模式",
                "编辑模式",
                "可视化编辑模式",
                "分栏预览模式",
                "阅读模式",
                "侧边栏",
                "文件",
                "大纲",
                "筛选文件",
                "主题",
                "专注模式",
                "打字机模式",
                "代码行号",
                "首选项",
            ],
            search: ["查找", "替换", "下一个/上一个匹配"],
            tables: ["格式化表格", "添加/删除行", "移动行", "添加/删除列"],
            export: ["HTML", "纯 HTML", "PDF", "LaTeX", "DOCX", "PNG", "JPEG"],
        },
        Language::ZhHant => ShortcutLabels {
            sections: ["檔案", "分頁", "編輯", "檢視", "搜尋", "表格", "匯出"],
            files: ["新增", "開啟", "儲存", "另存新檔", "結束"],
            tabs: ["在新分頁中開啟", "關閉分頁", "下一個/上一個分頁"],
            editing: [
                "粗體",
                "斜體",
                "行內程式碼",
                "連結",
                "圖片",
                "標題",
                "復原/重做",
            ],
            view: [
                "切換檢視模式",
                "編輯模式",
                "視覺化編輯模式",
                "分割預覽模式",
                "閱讀模式",
                "側邊欄",
                "檔案",
                "大綱",
                "篩選檔案",
                "主題",
                "專注模式",
                "打字機模式",
                "程式碼行號",
                "偏好設定",
            ],
            search: ["尋找", "取代", "下一個/上一個符合項目"],
            tables: ["格式化表格", "新增/刪除列", "移動列", "新增/刪除欄"],
            export: ["HTML", "純 HTML", "PDF", "LaTeX", "DOCX", "PNG", "JPEG"],
        },
        Language::Ja => ShortcutLabels {
            sections: [
                "ファイル",
                "タブ",
                "編集",
                "表示",
                "検索",
                "表",
                "エクスポート",
            ],
            files: ["新規作成", "開く", "保存", "名前を付けて保存", "終了"],
            tabs: ["新しいタブで開く", "タブを閉じる", "次/前のタブ"],
            editing: [
                "太字",
                "斜体",
                "インラインコード",
                "リンク",
                "画像",
                "見出し",
                "元に戻す/やり直す",
            ],
            view: [
                "表示モード切替",
                "編集モード",
                "ビジュアル編集モード",
                "分割プレビューモード",
                "閲覧モード",
                "サイドバー",
                "ファイル",
                "アウトライン",
                "ファイル絞り込み",
                "テーマ",
                "集中モード",
                "タイプライターモード",
                "コード行番号",
                "設定",
            ],
            search: ["検索", "置換", "次/前の一致"],
            tables: ["表の整形", "行の追加/削除", "行の移動", "列の追加/削除"],
            export: [
                "HTML",
                "プレーンHTML",
                "PDF",
                "LaTeX",
                "DOCX",
                "PNG",
                "JPEG",
            ],
        },
        Language::Fr => ShortcutLabels {
            sections: [
                "Fichiers",
                "Onglets",
                "Edition",
                "Affichage",
                "Recherche",
                "Tableaux",
                "Exportation",
            ],
            files: [
                "Nouveau",
                "Ouvrir",
                "Enregistrer",
                "Enregistrer sous",
                "Quitter",
            ],
            tabs: [
                "Ouvrir dans un nouvel onglet",
                "Fermer l'onglet",
                "Onglet suivant/precedent",
            ],
            editing: [
                "Gras",
                "Italique",
                "Code en ligne",
                "Lien",
                "Image",
                "Titres",
                "Annuler/Retablir",
            ],
            view: [
                "Changer le mode",
                "Mode edition",
                "Mode edition visuelle",
                "Mode apercu fractionne",
                "Mode lecture",
                "Barre laterale",
                "Fichiers",
                "Plan",
                "Filtrer les fichiers",
                "Theme",
                "Mode concentre",
                "Mode machine a ecrire",
                "Numeros de ligne",
                "Preferences",
            ],
            search: [
                "Rechercher",
                "Remplacer",
                "Correspondance suivante/precedente",
            ],
            tables: [
                "Formater le tableau",
                "Ajouter/Supprimer une ligne",
                "Deplacer la ligne",
                "Ajouter/Supprimer une colonne",
            ],
            export: ["HTML", "HTML simple", "PDF", "LaTeX", "DOCX", "PNG", "JPEG"],
        },
        Language::De => ShortcutLabels {
            sections: [
                "Dateien",
                "Tabs",
                "Bearbeitung",
                "Ansicht",
                "Suche",
                "Tabellen",
                "Export",
            ],
            files: ["Neu", "Oeffnen", "Speichern", "Speichern unter", "Beenden"],
            tabs: [
                "In neuem Tab oeffnen",
                "Tab schliessen",
                "Naechster/Vorheriger Tab",
            ],
            editing: [
                "Fett",
                "Kursiv",
                "Inline-Code",
                "Link",
                "Bild",
                "Ueberschriften",
                "Rueckgaengig/Wiederholen",
            ],
            view: [
                "Ansichtsmodus wechseln",
                "Bearbeitungsmodus",
                "Visueller Bearbeitungsmodus",
                "Geteilter Vorschaumodus",
                "Lesemodus",
                "Seitenleiste",
                "Dateien",
                "Gliederung",
                "Dateien filtern",
                "Design",
                "Fokusmodus",
                "Schreibmaschinenmodus",
                "Zeilennummern",
                "Einstellungen",
            ],
            search: ["Suchen", "Ersetzen", "Naechster/Vorheriger Treffer"],
            tables: [
                "Tabelle formatieren",
                "Zeile hinzufuegen/loeschen",
                "Zeile verschieben",
                "Spalte hinzufuegen/loeschen",
            ],
            export: [
                "HTML",
                "Einfaches HTML",
                "PDF",
                "LaTeX",
                "DOCX",
                "PNG",
                "JPEG",
            ],
        },
        Language::Es => ShortcutLabels {
            sections: [
                "Archivos", "Pestanas", "Edicion", "Ver", "Busqueda", "Tablas", "Exportar",
            ],
            files: ["Nuevo", "Abrir", "Guardar", "Guardar como", "Salir"],
            tabs: [
                "Abrir en nueva pestana",
                "Cerrar pestana",
                "Pestana siguiente/anterior",
            ],
            editing: [
                "Negrita",
                "Cursiva",
                "Codigo en linea",
                "Enlace",
                "Imagen",
                "Encabezados",
                "Deshacer/Rehacer",
            ],
            view: [
                "Cambiar modo de vista",
                "Modo edicion",
                "Modo edicion visual",
                "Modo vista previa dividida",
                "Modo lectura",
                "Barra lateral",
                "Archivos",
                "Esquema",
                "Filtrar archivos",
                "Tema",
                "Modo concentracion",
                "Modo maquina de escribir",
                "Numeros de linea",
                "Preferencias",
            ],
            search: ["Buscar", "Reemplazar", "Coincidencia siguiente/anterior"],
            tables: [
                "Formatear tabla",
                "Agregar/Eliminar fila",
                "Mover fila",
                "Agregar/Eliminar columna",
            ],
            export: ["HTML", "HTML simple", "PDF", "LaTeX", "DOCX", "PNG", "JPEG"],
        },
    }
}

/// Localised, human-readable name of a sidebar tab (used in status messages
/// and the preferences summary).
pub fn sidebar_tab_label(lang: Language, tab: SidebarTab) -> &'static str {
    match (lang, tab) {
        (Language::En, SidebarTab::Files) => "files",
        (Language::En, SidebarTab::Outline) => "outline",
        (Language::ZhHans, SidebarTab::Files) => "文件",
        (Language::ZhHans, SidebarTab::Outline) => "大纲",
        (Language::ZhHant, SidebarTab::Files) => "檔案",
        (Language::ZhHant, SidebarTab::Outline) => "大綱",
        (Language::Ja, SidebarTab::Files) => "ファイル",
        (Language::Ja, SidebarTab::Outline) => "アウトライン",
        (Language::Fr, SidebarTab::Files) => "fichiers",
        (Language::Fr, SidebarTab::Outline) => "plan",
        (Language::De, SidebarTab::Files) => "Dateien",
        (Language::De, SidebarTab::Outline) => "Gliederung",
        (Language::Es, SidebarTab::Files) => "archivos",
        (Language::Es, SidebarTab::Outline) => "esquema",
    }
}

// ---------------------------------------------------------------------------
// English
// ---------------------------------------------------------------------------

fn en(msg: Msg) -> &'static str {
    match msg {
        Msg::MenuFile => "File",
        Msg::MenuEdit => "Edit",
        Msg::MenuView => "View",
        Msg::MenuFormat => "Format",
        Msg::MenuExport => "Export",
        Msg::MenuHelp => "Help",

        Msg::ItemNew => "New",
        Msg::ItemOpen => "Open",
        Msg::ItemOpenFolder => "Open Folder",
        Msg::ItemSave => "Save",
        Msg::ItemSaveAs => "Save As",
        Msg::ItemNewTab => "New Tab",
        Msg::ItemOpenInNewTab => "Open in New Tab",
        Msg::ItemCloseTab => "Close Tab",
        Msg::ItemNextTab => "Next Tab",
        Msg::ItemPrevTab => "Previous Tab",
        Msg::ItemExit => "Exit",

        Msg::ItemUndo => "Undo",
        Msg::ItemRedo => "Redo",
        Msg::ItemCopy => "Copy",
        Msg::ItemCut => "Cut",
        Msg::ItemPaste => "Paste",
        Msg::ItemSelectAll => "Select All",
        Msg::ItemPreviewCopyPlain => "Copy as Plain Text",
        Msg::ItemPreviewCopyMarkdown => "Copy as Markdown",
        Msg::ItemPreviewCopyHtml => "Copy as HTML",
        Msg::ItemPreviewSelectAll => "Select All",
        Msg::ItemPreviewCopyLinkAddress => "Copy Link Address",

        Msg::ItemToggleView => "Cycle View Mode",
        Msg::ItemEditMode => "Edit Mode",
        Msg::ItemVisualEditMode => "Visual Edit Mode",
        Msg::ItemSplitPreviewMode => "Split Preview Mode",
        Msg::ItemReadMode => "Read Mode",
        Msg::ItemToggleSidebar => "Toggle Sidebar",
        Msg::ItemFiles => "Files",
        Msg::ItemOutline => "Outline",
        Msg::ItemFocusMode => "Focus Mode",
        Msg::ItemTypewriterMode => "Typewriter Mode",
        Msg::ItemCodeLineNumbers => "Code Line Numbers",
        Msg::ItemFind => "Find",
        Msg::ItemReplace => "Replace",
        Msg::ItemFindNext => "Find Next",
        Msg::ItemFindPrevious => "Find Previous",
        Msg::ItemCycleTheme => "Cycle Theme",
        Msg::ItemLanguage => "Language",

        Msg::ItemBold => "Bold",
        Msg::ItemItalic => "Italic",
        Msg::ItemInlineCode => "Inline Code",
        Msg::ItemLink => "Link",
        Msg::ItemImage => "Image",
        Msg::ItemH1 => "H1",
        Msg::ItemH2 => "H2",
        Msg::ItemH3 => "H3",
        Msg::ItemH4 => "H4",
        Msg::ItemH5 => "H5",
        Msg::ItemH6 => "H6",
        Msg::ItemBullets => "Bullets",
        Msg::ItemNumbers => "Numbers",
        Msg::ItemTask => "Task",
        Msg::ItemQuote => "Quote",
        Msg::ItemCodeFence => "Code Fence",
        Msg::ItemFormatTable => "Format Table",
        Msg::ItemAddTableRow => "Add Table Row",
        Msg::ItemDeleteTableRow => "Delete Table Row",
        Msg::ItemMoveRowUp => "Move Row Up",
        Msg::ItemMoveRowDown => "Move Row Down",
        Msg::ItemAddTableColumn => "Add Table Column",
        Msg::ItemDeleteTableColumn => "Delete Table Column",

        Msg::ItemExportHtml => "Export HTML",
        Msg::ItemExportPlainHtml => "Export Plain HTML",
        Msg::ItemExportPdf => "Export PDF",
        Msg::ItemExportLatex => "Export LaTeX",
        Msg::ItemExportDocx => "Export DOCX",
        Msg::ItemExportPng => "Export PNG",
        Msg::ItemExportJpeg => "Export JPEG",

        Msg::ItemPreferences => "Preferences",
        Msg::ItemResetPreferences => "Reset Preferences",
        Msg::ItemKeyboardShortcuts => "Keyboard Shortcuts",
        Msg::ItemAboutMarkion => "About Markion",

        Msg::LabelEditor => "Editor",
        Msg::LabelPreview => "Preview",
        Msg::LabelFiles => "Files",
        Msg::LabelOutline => "Outline",
        Msg::LabelTable => "Table",
        Msg::LabelImageAlt => "Alt text",
        Msg::LabelImageDestination => "Destination",
        Msg::LabelImageTitle => "Title",

        Msg::SearchFind => "Find",
        Msg::SearchReplace => "Replace",
        Msg::SearchPrev => "Prev",
        Msg::SearchNext => "Next",
        Msg::SearchAll => "All",
        Msg::SearchLiteral => "Text",
        Msg::SearchRegexMark => ".*",
        Msg::SearchCaseInsensitiveMark => "aa",
        Msg::SearchCaseSensitiveMark => "Aa",
        Msg::SearchProgress => "{0}/{1}",

        Msg::StatusReady => "Ready",
        Msg::StatusRecoveryAvailable => "Recovery available",
        Msg::StatusRecoveredDocument => "Recovered unsaved document",
        Msg::StatusRecoveryDiscarded => "Recovery discarded",
        Msg::StatusWaitingFileTreeConfirm => "Waiting for file tree confirmation...",
        Msg::StatusOpenCanceled => "Open canceled",
        Msg::StatusOpenFolderCanceled => "Open Folder canceled",
        Msg::StatusJumpedToHeading => "Jumped to heading",
        Msg::StatusNewDocument => "New document",
        Msg::StatusOpening => "Opening...",
        Msg::StatusOpeningFolder => "Opening folder...",
        Msg::StatusChoosingSaveLocation => "Choosing save location...",
        Msg::StatusEditMode => "Edit mode",
        Msg::StatusVisualEditMode => "Visual edit mode",
        Msg::StatusSplitPreviewMode => "Split preview mode",
        Msg::StatusReadMode => "Read mode",
        Msg::StatusSidebarShown => "Sidebar shown",
        Msg::StatusSidebarHidden => "Sidebar hidden",
        Msg::StatusOutlineShown => "Outline shown",
        Msg::StatusFileTreeShown => "File tree shown",
        Msg::StatusFilteringFiles => "Filtering files",
        Msg::StatusFileTreeRefreshed => "File tree refreshed",
        Msg::StatusSelectTreeEntryFirst => "Select a file tree entry first",
        Msg::StatusSaveBeforeRename => "Save the active document before renaming it",
        Msg::StatusWaitingDeleteConfirm => "Waiting for delete confirmation...",
        Msg::StatusDeleteCanceled => "Delete canceled",
        Msg::StatusAboutMarkion => "About Markion",
        Msg::StatusKeyboardShortcuts => "Keyboard shortcuts",
        Msg::StatusUndo => "Undo",
        Msg::StatusNothingToUndo => "Nothing to undo",
        Msg::StatusRedo => "Redo",
        Msg::StatusNothingToRedo => "Nothing to redo",
        Msg::StatusNoFormattingChange => "No formatting change",
        Msg::StatusNoTableAtCursor => "No table at cursor",
        Msg::StatusTableAlreadyFormatted => "Table already formatted",
        Msg::StatusWaitingConfirm => "Waiting for confirmation...",
        Msg::StatusCanceled => "Canceled",
        Msg::StatusExitingMarkion => "Exiting Markion",
        Msg::StatusWaitingExitConfirm => "Waiting for exit confirmation...",
        Msg::StatusExitCanceled => "Exit canceled",
        Msg::StatusWaitingQuitConfirm => "Waiting for quit confirmation...",
        Msg::StatusEditingFindQuery => "Editing find query",
        Msg::StatusEditingReplacement => "Editing replacement",
        Msg::StatusEditing => "Editing",
        Msg::StatusIndentedSelection => "Indented selection",
        Msg::StatusOutdentedSelection => "Outdented selection",
        Msg::StatusNothingToIndent => "Nothing to indent",
        Msg::StatusNothingToOutdent => "Nothing to outdent",
        Msg::StatusNoEdit => "No edit",
        Msg::StatusClipboardEmpty => "Clipboard is empty",
        Msg::StatusCopiedSelection => "Copied selection",
        Msg::StatusCopiedPreviewPlain => "Copied selection as plain text",
        Msg::StatusCopiedPreviewMarkdown => "Copied selection as Markdown",
        Msg::StatusCopiedPreviewHtml => "Copied selection as HTML",
        Msg::StatusCopiedLinkAddress => "Copied link address",
        Msg::StatusPreviewSelectedAll => "Selected all preview text",
        Msg::StatusNothingToCopy => "Nothing selected to copy",
        Msg::StatusCutSelection => "Cut selection",
        Msg::StatusNothingToCut => "Nothing selected to cut",
        Msg::StatusComposing => "Composing",
        Msg::StatusSelectedTreeEntry => "Selected file tree entry",
        Msg::StatusNoMatchSelected => "No match selected",
        Msg::StatusReplacedCurrent => "Replaced current match",
        Msg::StatusNoMatchesToReplace => "No matches to replace",
        Msg::StatusFindQueryEmpty => "Find query is empty",
        Msg::StatusNoMatches => "No matches",

        Msg::StatusFmtBold => "Bold",
        Msg::StatusFmtItalic => "Italic",
        Msg::StatusFmtInlineCode => "Inline code",
        Msg::StatusFmtLink => "Insert link",
        Msg::StatusFmtImage => "Insert image",
        Msg::StatusFmtHeading => "Heading {0}",
        Msg::StatusFmtBulletedList => "Bulleted list",
        Msg::StatusFmtNumberedList => "Numbered list",
        Msg::StatusFmtTaskList => "Task list",
        Msg::StatusFmtBlockQuote => "Block quote",
        Msg::StatusFmtCodeBlock => "Code block",
        Msg::StatusFmtFormatTable => "Format table",
        Msg::StatusFmtAddRow => "Add table row",
        Msg::StatusFmtDeleteRow => "Delete table row",
        Msg::StatusFmtMoveRowUp => "Move table row up",
        Msg::StatusFmtMoveRowDown => "Move table row down",
        Msg::StatusFmtAddColumn => "Add table column",
        Msg::StatusFmtDeleteColumn => "Delete table column",
        Msg::StatusFocusModeOn => "Focus mode on",
        Msg::StatusFocusModeOff => "Focus mode off",
        Msg::StatusTypewriterModeOn => "Typewriter mode on",
        Msg::StatusTypewriterModeOff => "Typewriter mode off",
        Msg::StatusCodeLineNumbersOn => "Code line numbers on",
        Msg::StatusCodeLineNumbersOff => "Code line numbers off",
        Msg::StatusRegexFind => "Regex find",
        Msg::StatusLiteralFind => "Literal find",
        Msg::StatusPreferences => "Preferences",
        Msg::StatusWaitingPreferenceResetConfirm => "Waiting for preference reset confirmation...",
        Msg::StatusPreferencesReset => "Preferences reset",
        Msg::StatusPreviewAdaptiveWidthOn => "Preview adaptive width on",
        Msg::StatusPreviewAdaptiveWidthOff => "Preview adaptive width off",
        Msg::StatusSyncScrollOn => "Sync scroll on",
        Msg::StatusSyncScrollOff => "Sync scroll off",
        Msg::StatusPreferenceResetCanceled => "Preference reset canceled",
        Msg::StatusLanguageSet => "Language set",

        Msg::StatusRecoveryFailed => "Recovery failed: {0}",
        Msg::StatusOpened => "Opened {0}",
        Msg::StatusOpenFailed => "Open failed: {0}",
        Msg::StatusOpenedFolder => "Opened folder {0}",
        Msg::StatusOpenFolderFailed => "Open folder failed: {0}",
        Msg::StatusAutoSaved => "Auto-saved {0}",
        Msg::StatusRecoverySaved => "Recovery saved {0}",
        Msg::StatusAutoSaveFailed => "Auto-save failed: {0}",
        Msg::StatusFindFailed => "Find failed: {0}",
        Msg::StatusPreferencesSaveFailed => "Preferences save failed: {0}",
        Msg::StatusSampleThemeSaveFailed => "Sample theme save failed: {0}",
        Msg::StatusSaved => "Saved {0}",
        Msg::StatusSaveFailed => "Save failed: {0}",
        Msg::StatusSaveCanceled => "Save canceled",
        Msg::StatusExportCanceled => "Export canceled",
        Msg::StatusChoosingExportLocation => "Choosing .{0} export location...",
        Msg::StatusExported => "Exported {0}",
        Msg::StatusExportedEngine => "Exported {0} (pandoc engine)",
        Msg::StatusExportedBuiltin => {
            "Exported {0} (built-in writer — install pandoc for richer output)"
        }
        Msg::StatusExportFailed => "Export failed: {0}",
        Msg::StatusCreated => "Created {0}",
        Msg::StatusCreateFileFailed => "Create file failed: {0}",
        Msg::StatusCreateFolderFailed => "Create folder failed: {0}",
        Msg::StatusRenamedTo => "Renamed to {0}",
        Msg::StatusRenameFailed => "Rename failed: {0}",
        Msg::StatusDeleted => "Deleted {0}",
        Msg::StatusDeleteFailed => "Delete failed: {0}",
        Msg::StatusShownInFileManager => "Shown in system file manager: {0}",
        Msg::StatusShowInFileManagerFailed => "Show in system file manager failed: {0}",
        Msg::StatusTheme => "Theme: {0}",
        Msg::StatusMatches => "{0} matches",
        Msg::StatusReplaceFailed => "Replace failed: {0}",
        Msg::StatusReplacedMatches => "Replaced {0} matches",
        Msg::StatusMatchPosition => "Match {0} of {1} at {2}:{3}",
        Msg::StatusFilesVisible => "{0} files visible",
        Msg::StatusFileMatches => "{0} file matches",

        Msg::DialogButtonOk => "OK",
        Msg::DialogButtonCancel => "Cancel",
        Msg::DialogButtonDiscard => "Discard",
        Msg::DialogButtonDelete => "Delete",
        Msg::DialogButtonReset => "Reset",
        Msg::DialogButtonRestore => "Restore",
        Msg::DialogButtonExitWithoutSaving => "Exit Without Saving",

        Msg::DialogAboutTitle => "About Markion",
        Msg::DialogAboutDetail => {
            "Version: {0}\n\nA local-first Markdown editor built with Rust and GPUI.\n\nGitHub: {1}"
        }
        Msg::DialogShortcutsTitle => "Keyboard Shortcuts",
        Msg::DialogPreferencesTitle => "Preferences",
        Msg::DialogPreferencesDetail => PREFERENCES_DETAIL_EN,
        Msg::DialogRestoreTitle => "Restore unsaved document?",
        Msg::DialogRestoreDetail => "Markion found an unsaved recovery file:\n{0}",
        Msg::DialogDiscardTitle => "Discard unsaved changes?",
        Msg::DialogDiscardNewDetail => "Create a new document without saving the current changes.",
        Msg::DialogDiscardOpenDetail => "Open another document without saving the current changes.",
        Msg::DialogDiscardOpenTreeDetail => "Open {0} without saving current changes.",
        Msg::DialogExitTitle => "Exit Markion without saving?",
        Msg::DialogExitDetail => "Unsaved changes will be lost.",
        Msg::DialogDeleteTitle => "Delete selected file tree entry?",
        Msg::DialogDeleteDetail => "Delete {0} from disk.",
        Msg::DialogDeleteFolderRecursiveTitle => "Delete folder and all contents?",
        Msg::DialogDeleteFolderRecursiveDetail => {
            "The folder {0} and everything inside it will be permanently removed."
        }
        Msg::DialogResetTitle => "Reset preferences?",
        Msg::DialogResetDetail => {
            "Theme, focus mode, typewriter mode, code line-number, preview width, and sidebar settings will return to defaults."
        }
        Msg::PromptOpenMarkdown => "Open Markdown",
        Msg::PromptOpenFolder => "Open Folder",
        Msg::FileTypeMarkdown => "Markdown document",
        Msg::FileTypeStyledHtml => "HTML document",
        Msg::FileTypePlainHtml => "Plain HTML document",
        Msg::FileTypePdf => "PDF document",
        Msg::FileTypeLatex => "LaTeX document",
        Msg::FileTypeDocx => "Word document (DOCX)",
        Msg::FileTypePng => "PNG image",
        Msg::FileTypeJpeg => "JPEG image",

        Msg::SummaryFilesUnit => "files",
        Msg::SummaryMatchesUnit => "matches",

        Msg::FileTreeFilterPlaceholder => "Filter files",
        Msg::FileTreeFilterActive => "Filter: {0}",
        Msg::FileTreeWorkspaceFallback => "Workspace",
        Msg::FileTreeMoreHidden => "... {0} more (filter to narrow)",
        Msg::FileTreeEmptyState => "Open a Markdown file to see it listed here.",
        Msg::FileTreeContextOpen => "Open",
        Msg::FileTreeContextOpenInNewTab => "Open in New Tab",
        Msg::FileTreeContextCreateFile => "New File",
        Msg::FileTreeContextCreateFolder => "New Folder",
        Msg::FileTreeContextRename => "Rename",
        Msg::FileTreeContextDelete => "Delete",
        Msg::FileTreeContextShowInFileManager => "Show in System File Manager",
        Msg::FileTreeContextRefresh => "Refresh",
        Msg::FileTreeContextFilterFiles => "Filter Files",
        Msg::FileTreeNamePromptLabel => "Name",
        Msg::StatusNamingEntry => "Type a name and press Enter (Esc to cancel)",
        Msg::StatusNameRequired => "Name cannot be empty",

        Msg::PrefOn => "on",
        Msg::PrefOff => "off",
        Msg::PrefSidebarHidden => "hidden",
        Msg::CustomThemeLabel => "{0} (custom)",

        Msg::PrefPanelTitle => "Preferences",
        Msg::PrefPanelThemeSection => "Theme",
        Msg::PrefPanelLanguageSection => "Language",
        Msg::PrefPanelOtherSection => "Other",
        Msg::PrefPanelFocusMode => "Focus mode",
        Msg::PrefPanelTypewriterMode => "Typewriter mode",
        Msg::PrefPanelCodeLineNumbers => "Code line numbers",
        Msg::PrefPanelPreviewAdaptiveWidth => "Preview adaptive width",
        Msg::PrefPanelSyncScroll => "Sync scroll",
        Msg::PrefPanelSidebar => "Sidebar",
        Msg::PrefPanelHeadingMenu => "Heading menu",
        Msg::PrefPanelHeadingMenuThree => "H1–H5",
        Msg::PrefPanelHeadingMenuSix => "H1–H6",
        Msg::PrefPanelClose => "Close",
        Msg::DiagramLoading => "Rendering diagram…",
        Msg::DiagramUnsupported => "This diagram format is not supported.",
        Msg::DiagramInputTooLarge => "The diagram source exceeds the size limit.",
        Msg::DiagramInvalidSource => "The diagram source is invalid.",
        Msg::DiagramUnsafeOutput => "The diagram output was blocked for safety.",
        Msg::DiagramRenderFailed => "Diagram rendering failed.",
        Msg::MathRendering => "Rendering formula…",
        Msg::MathInvalid => "The formula is invalid.",
        Msg::MathTooLarge => "The formula exceeds the rendering size limit.",
        Msg::MathUnsupported => "The formula uses unsupported notation or glyphs.",
        Msg::MathRenderFailed => "Formula rendering failed.",
        Msg::TitleModified => "Modified",
        Msg::TitleSaved => "Saved",
    }
}

// Preferences summary body. Placeholders:
//   {0}=theme {1}=focus {2}=typewriter {3}=line numbers
//   {4}=preview adaptive width {5}=sidebar {6}=prefs path
//   {7}=themes dir {8}=custom theme count
const PREFERENCES_DETAIL_EN: &str = "Theme: {0}\nFocus mode: {1}\nTypewriter mode: {2}\nCode line numbers: {3}\nPreview adaptive width: {4}\nSidebar: {5}\n\nPreferences: {6}\nCustom themes: {7}\nInstalled custom themes: {8}";

// ---------------------------------------------------------------------------
// Japanese
// ---------------------------------------------------------------------------

fn ja(msg: Msg) -> &'static str {
    match msg {
        Msg::MenuFile => "ファイル",
        Msg::MenuEdit => "編集",
        Msg::MenuView => "表示",
        Msg::MenuFormat => "書式",
        Msg::MenuExport => "エクスポート",
        Msg::MenuHelp => "ヘルプ",

        Msg::ItemNew => "新規作成",
        Msg::ItemOpen => "開く",
        Msg::ItemOpenFolder => "フォルダーを開く",
        Msg::ItemSave => "保存",
        Msg::ItemSaveAs => "名前を付けて保存",
        Msg::ItemNewTab => "新しいタブ",
        Msg::ItemOpenInNewTab => "新しいタブで開く",
        Msg::ItemCloseTab => "タブを閉じる",
        Msg::ItemNextTab => "次のタブ",
        Msg::ItemPrevTab => "前のタブ",
        Msg::ItemExit => "終了",

        Msg::ItemUndo => "元に戻す",
        Msg::ItemRedo => "やり直す",
        Msg::ItemCopy => "コピー",
        Msg::ItemCut => "切り取り",
        Msg::ItemPaste => "貼り付け",
        Msg::ItemSelectAll => "すべて選択",
        Msg::ItemPreviewCopyPlain => "プレーンテキストとしてコピー",
        Msg::ItemPreviewCopyMarkdown => "Markdownとしてコピー",
        Msg::ItemPreviewCopyHtml => "HTMLとしてコピー",
        Msg::ItemPreviewSelectAll => "すべて選択",
        Msg::ItemPreviewCopyLinkAddress => "リンクアドレスをコピー",

        Msg::ItemToggleView => "表示モード切替",
        Msg::ItemEditMode => "編集モード",
        Msg::ItemVisualEditMode => "ビジュアル編集モード",
        Msg::ItemSplitPreviewMode => "分割プレビューモード",
        Msg::ItemReadMode => "閲覧モード",
        Msg::ItemToggleSidebar => "サイドバー切替",
        Msg::ItemFiles => "ファイル",
        Msg::ItemOutline => "アウトライン",
        Msg::ItemFocusMode => "集中モード",
        Msg::ItemTypewriterMode => "タイプライターモード",
        Msg::ItemCodeLineNumbers => "コード行番号",
        Msg::ItemFind => "検索",
        Msg::ItemReplace => "置換",
        Msg::ItemFindNext => "次を検索",
        Msg::ItemFindPrevious => "前を検索",
        Msg::ItemCycleTheme => "テーマ切替",
        Msg::ItemLanguage => "言語",

        Msg::ItemBold => "太字",
        Msg::ItemItalic => "斜体",
        Msg::ItemInlineCode => "インラインコード",
        Msg::ItemLink => "リンク",
        Msg::ItemImage => "画像",
        Msg::ItemH1 => "H1",
        Msg::ItemH2 => "H2",
        Msg::ItemH3 => "H3",
        Msg::ItemH4 => "H4",
        Msg::ItemH5 => "H5",
        Msg::ItemH6 => "H6",
        Msg::ItemBullets => "箇条書き",
        Msg::ItemNumbers => "番号付きリスト",
        Msg::ItemTask => "タスク",
        Msg::ItemQuote => "引用",
        Msg::ItemCodeFence => "コードブロック",
        Msg::ItemFormatTable => "表の整形",
        Msg::ItemAddTableRow => "行を追加",
        Msg::ItemDeleteTableRow => "行を削除",
        Msg::ItemMoveRowUp => "行を上へ移動",
        Msg::ItemMoveRowDown => "行を下へ移動",
        Msg::ItemAddTableColumn => "列を追加",
        Msg::ItemDeleteTableColumn => "列を削除",

        Msg::ItemExportHtml => "HTML出力",
        Msg::ItemExportPlainHtml => "プレーンHTML出力",
        Msg::ItemExportPdf => "PDF出力",
        Msg::ItemExportLatex => "LaTeX出力",
        Msg::ItemExportDocx => "DOCX出力",
        Msg::ItemExportPng => "PNG出力",
        Msg::ItemExportJpeg => "JPEG出力",

        Msg::ItemPreferences => "設定",
        Msg::ItemResetPreferences => "設定をリセット",
        Msg::ItemKeyboardShortcuts => "キーボードショートカット",
        Msg::ItemAboutMarkion => "Markionについて",

        Msg::LabelEditor => "エディタ",
        Msg::LabelPreview => "プレビュー",
        Msg::LabelFiles => "ファイル",
        Msg::LabelOutline => "アウトライン",
        Msg::LabelTable => "表",
        Msg::LabelImageAlt => "代替テキスト",
        Msg::LabelImageDestination => "URL",
        Msg::LabelImageTitle => "タイトル",

        Msg::SearchFind => "検索",
        Msg::SearchReplace => "置換",
        Msg::SearchPrev => "前へ",
        Msg::SearchNext => "次へ",
        Msg::SearchAll => "すべて",
        Msg::SearchLiteral => "テキスト",
        Msg::SearchRegexMark => ".*",
        Msg::SearchCaseInsensitiveMark => "aa",
        Msg::SearchCaseSensitiveMark => "Aa",
        Msg::SearchProgress => "{0}/{1}",

        Msg::StatusReady => "準備完了",
        Msg::StatusRecoveryAvailable => "復元可能なデータがあります",
        Msg::StatusRecoveredDocument => "未保存の文書を復元しました",
        Msg::StatusRecoveryDiscarded => "復元データを破棄しました",
        Msg::StatusWaitingFileTreeConfirm => "ファイルツリーの確認待ち...",
        Msg::StatusOpenCanceled => "開くをキャンセルしました",
        Msg::StatusOpenFolderCanceled => "フォルダーを開く操作をキャンセルしました",
        Msg::StatusJumpedToHeading => "見出しにジャンプしました",
        Msg::StatusNewDocument => "新規文書",
        Msg::StatusOpening => "開いています...",
        Msg::StatusOpeningFolder => "フォルダーを開いています...",
        Msg::StatusChoosingSaveLocation => "保存先を選択中...",
        Msg::StatusEditMode => "編集モード",
        Msg::StatusVisualEditMode => "ビジュアル編集モード",
        Msg::StatusSplitPreviewMode => "分割プレビューモード",
        Msg::StatusReadMode => "閲覧モード",
        Msg::StatusSidebarShown => "サイドバーを表示しました",
        Msg::StatusSidebarHidden => "サイドバーを非表示にしました",
        Msg::StatusOutlineShown => "アウトラインを表示しました",
        Msg::StatusFileTreeShown => "ファイルツリーを表示しました",
        Msg::StatusFilteringFiles => "ファイルを絞り込み中",
        Msg::StatusFileTreeRefreshed => "ファイルツリーを更新しました",
        Msg::StatusSelectTreeEntryFirst => "先にファイルツリーの項目を選択してください",
        Msg::StatusSaveBeforeRename => "名前を変更する前にアクティブな文書を保存してください",
        Msg::StatusWaitingDeleteConfirm => "削除の確認待ち...",
        Msg::StatusDeleteCanceled => "削除をキャンセルしました",
        Msg::StatusAboutMarkion => "Markionについて",
        Msg::StatusKeyboardShortcuts => "キーボードショートカット",
        Msg::StatusUndo => "元に戻す",
        Msg::StatusNothingToUndo => "元に戻す操作はありません",
        Msg::StatusRedo => "やり直す",
        Msg::StatusNothingToRedo => "やり直す操作はありません",
        Msg::StatusNoFormattingChange => "書式の変更はありません",
        Msg::StatusNoTableAtCursor => "カーソル位置に表がありません",
        Msg::StatusTableAlreadyFormatted => "表は既に整形されています",
        Msg::StatusWaitingConfirm => "確認待ち...",
        Msg::StatusCanceled => "キャンセルしました",
        Msg::StatusExitingMarkion => "Markionを終了しています",
        Msg::StatusWaitingExitConfirm => "終了の確認待ち...",
        Msg::StatusExitCanceled => "終了をキャンセルしました",
        Msg::StatusWaitingQuitConfirm => "終了の確認待ち...",
        Msg::StatusEditingFindQuery => "検索クエリを編集中",
        Msg::StatusEditingReplacement => "置換テキストを編集中",
        Msg::StatusEditing => "編集中",
        Msg::StatusIndentedSelection => "選択範囲をインデントしました",
        Msg::StatusOutdentedSelection => "選択範囲のインデントを解除しました",
        Msg::StatusNothingToIndent => "インデントする項目がありません",
        Msg::StatusNothingToOutdent => "インデント解除する項目がありません",
        Msg::StatusNoEdit => "編集操作がありません",
        Msg::StatusClipboardEmpty => "クリップボードが空です",
        Msg::StatusCopiedSelection => "選択範囲をコピーしました",
        Msg::StatusCopiedPreviewPlain => "選択範囲をプレーンテキストとしてコピーしました",
        Msg::StatusCopiedPreviewMarkdown => "選択範囲をMarkdownとしてコピーしました",
        Msg::StatusCopiedPreviewHtml => "選択範囲をHTMLとしてコピーしました",
        Msg::StatusCopiedLinkAddress => "リンクアドレスをコピーしました",
        Msg::StatusPreviewSelectedAll => "プレビューのテキストをすべて選択しました",
        Msg::StatusNothingToCopy => "コピーする項目が選択されていません",
        Msg::StatusCutSelection => "選択範囲を切り取りました",
        Msg::StatusNothingToCut => "切り取る項目が選択されていません",
        Msg::StatusComposing => "入力中",
        Msg::StatusSelectedTreeEntry => "ファイルツリーの項目を選択しました",
        Msg::StatusNoMatchSelected => "一致する項目が選択されていません",
        Msg::StatusReplacedCurrent => "現在の一致を置換しました",
        Msg::StatusNoMatchesToReplace => "置換する一致がありません",
        Msg::StatusFindQueryEmpty => "検索クエリが空です",
        Msg::StatusNoMatches => "一致するものはありません",

        Msg::StatusFmtBold => "太字",
        Msg::StatusFmtItalic => "斜体",
        Msg::StatusFmtInlineCode => "インラインコード",
        Msg::StatusFmtLink => "リンクを挿入",
        Msg::StatusFmtImage => "画像を挿入",
        Msg::StatusFmtHeading => "見出し {0}",
        Msg::StatusFmtBulletedList => "箇条書きリスト",
        Msg::StatusFmtNumberedList => "番号付きリスト",
        Msg::StatusFmtTaskList => "タスクリスト",
        Msg::StatusFmtBlockQuote => "引用ブロック",
        Msg::StatusFmtCodeBlock => "コードブロック",
        Msg::StatusFmtFormatTable => "表を整形",
        Msg::StatusFmtAddRow => "表の行を追加",
        Msg::StatusFmtDeleteRow => "表の行を削除",
        Msg::StatusFmtMoveRowUp => "表の行を上へ移動",
        Msg::StatusFmtMoveRowDown => "表の行を下へ移動",
        Msg::StatusFmtAddColumn => "表の列を追加",
        Msg::StatusFmtDeleteColumn => "表の列を削除",
        Msg::StatusFocusModeOn => "集中モード オン",
        Msg::StatusFocusModeOff => "集中モード オフ",
        Msg::StatusTypewriterModeOn => "タイプライターモード オン",
        Msg::StatusTypewriterModeOff => "タイプライターモード オフ",
        Msg::StatusCodeLineNumbersOn => "コード行番号 オン",
        Msg::StatusCodeLineNumbersOff => "コード行番号 オフ",
        Msg::StatusRegexFind => "正規表現検索",
        Msg::StatusLiteralFind => "テキスト検索",
        Msg::StatusPreferences => "設定",
        Msg::StatusWaitingPreferenceResetConfirm => "設定リセットの確認待ち...",
        Msg::StatusPreferencesReset => "設定をリセットしました",
        Msg::StatusPreviewAdaptiveWidthOn => "プレビュー幅自動調整 オン",
        Msg::StatusPreviewAdaptiveWidthOff => "プレビュー幅自動調整 オフ",
        Msg::StatusSyncScrollOn => "同期スクロール オン",
        Msg::StatusSyncScrollOff => "同期スクロール オフ",
        Msg::StatusPreferenceResetCanceled => "設定リセットをキャンセルしました",
        Msg::StatusLanguageSet => "言語を設定しました",

        Msg::StatusRecoveryFailed => "復元に失敗しました: {0}",
        Msg::StatusOpened => "{0} を開きました",
        Msg::StatusOpenFailed => "開けませんでした: {0}",
        Msg::StatusOpenedFolder => "フォルダー {0} を開きました",
        Msg::StatusOpenFolderFailed => "フォルダーを開けませんでした: {0}",
        Msg::StatusAutoSaved => "{0} を自動保存しました",
        Msg::StatusRecoverySaved => "{0} を復元保存しました",
        Msg::StatusAutoSaveFailed => "自動保存に失敗しました: {0}",
        Msg::StatusFindFailed => "検索に失敗しました: {0}",
        Msg::StatusPreferencesSaveFailed => "設定の保存に失敗しました: {0}",
        Msg::StatusSampleThemeSaveFailed => "サンプルテーマの保存に失敗しました: {0}",
        Msg::StatusSaved => "{0} を保存しました",
        Msg::StatusSaveFailed => "保存に失敗しました: {0}",
        Msg::StatusSaveCanceled => "保存をキャンセルしました",
        Msg::StatusExportCanceled => "エクスポートをキャンセルしました",
        Msg::StatusChoosingExportLocation => ".{0} の出力先を選択中...",
        Msg::StatusExported => "{0} を出力しました",
        Msg::StatusExportedEngine => "{0} を出力しました（pandocエンジン）",
        Msg::StatusExportedBuiltin => {
            "{0} を出力しました（内蔵ライター — pandocをインストールするとより高品質な出力が可能です）"
        }
        Msg::StatusExportFailed => "エクスポートに失敗しました: {0}",
        Msg::StatusCreated => "{0} を作成しました",
        Msg::StatusCreateFileFailed => "ファイルの作成に失敗しました: {0}",
        Msg::StatusCreateFolderFailed => "フォルダの作成に失敗しました: {0}",
        Msg::StatusRenamedTo => "{0} に名前を変更しました",
        Msg::StatusRenameFailed => "名前の変更に失敗しました: {0}",
        Msg::StatusDeleted => "{0} を削除しました",
        Msg::StatusDeleteFailed => "削除に失敗しました: {0}",
        Msg::StatusShownInFileManager => "システムのファイルマネージャーで表示: {0}",
        Msg::StatusShowInFileManagerFailed => {
            "システムのファイルマネージャーでの表示に失敗しました: {0}"
        }
        Msg::StatusTheme => "テーマ: {0}",
        Msg::StatusMatches => "{0} 件の一致",
        Msg::StatusReplaceFailed => "置換に失敗しました: {0}",
        Msg::StatusReplacedMatches => "{0} 件の一致を置換しました",
        Msg::StatusMatchPosition => "一致 {0} / {1} (行 {2}:{3})",
        Msg::StatusFilesVisible => "{0} 個のファイルを表示中",
        Msg::StatusFileMatches => "{0} 件のファイル一致",

        Msg::DialogButtonOk => "OK",
        Msg::DialogButtonCancel => "キャンセル",
        Msg::DialogButtonDiscard => "破棄",
        Msg::DialogButtonDelete => "削除",
        Msg::DialogButtonReset => "リセット",
        Msg::DialogButtonRestore => "復元",
        Msg::DialogButtonExitWithoutSaving => "保存せずに終了",

        Msg::DialogAboutTitle => "Markionについて",
        Msg::DialogAboutDetail => {
            "バージョン: {0}\n\nRustとGPUIで構築されたローカルファーストのMarkdownエディタです。\n\nGitHub: {1}"
        }
        Msg::DialogShortcutsTitle => "キーボードショートカット",
        Msg::DialogPreferencesTitle => "設定",
        Msg::DialogPreferencesDetail => PREFERENCES_DETAIL_JA,
        Msg::DialogRestoreTitle => "未保存の文書を復元しますか？",
        Msg::DialogRestoreDetail => "Markionが未保存の復元ファイルを見つけました:\n{0}",
        Msg::DialogDiscardTitle => "未保存の変更を破棄しますか？",
        Msg::DialogDiscardNewDetail => "現在の変更を保存せずに新しい文書を作成します。",
        Msg::DialogDiscardOpenDetail => "現在の変更を保存せずに別の文書を開きます。",
        Msg::DialogDiscardOpenTreeDetail => "現在の変更を保存せずに {0} を開きます。",
        Msg::DialogExitTitle => "保存せずにMarkionを終了しますか？",
        Msg::DialogExitDetail => "未保存の変更は失われます。",
        Msg::DialogDeleteTitle => "選択したファイルツリーの項目を削除しますか？",
        Msg::DialogDeleteDetail => "{0} をディスクから削除します。",
        Msg::DialogDeleteFolderRecursiveTitle => "フォルダとその中身をすべて削除しますか？",
        Msg::DialogDeleteFolderRecursiveDetail => {
            "フォルダ {0} とその中のすべてのファイルが完全に削除されます。"
        }
        Msg::DialogResetTitle => "設定をリセットしますか？",
        Msg::DialogResetDetail => {
            "テーマ、集中モード、タイプライターモード、コード行番号、プレビュー幅、サイドバーの設定がデフォルトに戻ります。"
        }
        Msg::PromptOpenMarkdown => "Markdownを開く",
        Msg::PromptOpenFolder => "フォルダーを開く",
        Msg::FileTypeMarkdown => "Markdown文書",
        Msg::FileTypeStyledHtml => "HTML文書",
        Msg::FileTypePlainHtml => "プレーンHTML文書",
        Msg::FileTypePdf => "PDF文書",
        Msg::FileTypeLatex => "LaTeX文書",
        Msg::FileTypeDocx => "Word文書（DOCX）",
        Msg::FileTypePng => "PNG画像",
        Msg::FileTypeJpeg => "JPEG画像",

        Msg::SummaryFilesUnit => "ファイル",
        Msg::SummaryMatchesUnit => "件",

        Msg::FileTreeFilterPlaceholder => "ファイルを絞り込み",
        Msg::FileTreeFilterActive => "絞り込み: {0}",
        Msg::FileTreeWorkspaceFallback => "ワークスペース",
        Msg::FileTreeMoreHidden => "... 他 {0} 件（絞り込みで絞る）",
        Msg::FileTreeEmptyState => "Markdownファイルを開くとここに表示されます。",
        Msg::FileTreeContextOpen => "開く",
        Msg::FileTreeContextOpenInNewTab => "新しいタブで開く",
        Msg::FileTreeContextCreateFile => "新しいファイル",
        Msg::FileTreeContextCreateFolder => "新しいフォルダ",
        Msg::FileTreeContextRename => "名前を変更",
        Msg::FileTreeContextDelete => "削除",
        Msg::FileTreeContextShowInFileManager => "システムのファイルマネージャーで表示",
        Msg::FileTreeContextRefresh => "更新",
        Msg::FileTreeContextFilterFiles => "ファイルを絞り込み",
        Msg::FileTreeNamePromptLabel => "名前",
        Msg::StatusNamingEntry => "名前を入力してEnterを押してください（Escでキャンセル）",
        Msg::StatusNameRequired => "名前を入力してください",

        Msg::PrefOn => "オン",
        Msg::PrefOff => "オフ",
        Msg::PrefSidebarHidden => "非表示",
        Msg::CustomThemeLabel => "{0}（カスタム）",

        Msg::PrefPanelTitle => "設定",
        Msg::PrefPanelThemeSection => "テーマ",
        Msg::PrefPanelLanguageSection => "言語",
        Msg::PrefPanelOtherSection => "その他",
        Msg::PrefPanelFocusMode => "集中モード",
        Msg::PrefPanelTypewriterMode => "タイプライターモード",
        Msg::PrefPanelCodeLineNumbers => "コード行番号",
        Msg::PrefPanelPreviewAdaptiveWidth => "プレビュー幅自動調整",
        Msg::PrefPanelSidebar => "サイドバー",
        Msg::PrefPanelSyncScroll => "同期スクロール",
        Msg::PrefPanelHeadingMenu => "見出しメニュー",
        Msg::PrefPanelHeadingMenuThree => "H1–H5",
        Msg::PrefPanelHeadingMenuSix => "H1–H6",
        Msg::PrefPanelClose => "閉じる",
        Msg::DiagramLoading => "図を描画しています…",
        Msg::DiagramUnsupported => "この図形式はサポートされていません。",
        Msg::DiagramInputTooLarge => "図のソースがサイズ上限を超えています。",
        Msg::DiagramInvalidSource => "図のソースが無効です。",
        Msg::DiagramUnsafeOutput => "安全上の理由で図の出力をブロックしました。",
        Msg::DiagramRenderFailed => "図の描画に失敗しました。",
        Msg::MathRendering => "数式を描画しています…",
        Msg::MathInvalid => "数式が無効です。",
        Msg::MathTooLarge => "数式が描画サイズの上限を超えています。",
        Msg::MathUnsupported => "数式に未対応の記法または文字が含まれています。",
        Msg::MathRenderFailed => "数式の描画に失敗しました。",
        Msg::TitleModified => "変更あり",
        Msg::TitleSaved => "保存済み",
    }
}

const PREFERENCES_DETAIL_JA: &str = "テーマ: {0}\n集中モード: {1}\nタイプライターモード: {2}\nコード行番号: {3}\nプレビュー幅自動調整: {4}\nサイドバー: {5}\n\n設定ファイル: {6}\nカスタムテーマ: {7}\nインストール済みカスタムテーマ: {8}";

// ---------------------------------------------------------------------------
// French
// ---------------------------------------------------------------------------

fn fr(msg: Msg) -> &'static str {
    match msg {
        Msg::MenuFile => "Fichier",
        Msg::MenuEdit => "Édition",
        Msg::MenuView => "Affichage",
        Msg::MenuFormat => "Format",
        Msg::MenuExport => "Exporter",
        Msg::MenuHelp => "Aide",
        Msg::ItemNew => "Nouveau",
        Msg::ItemOpen => "Ouvrir",
        Msg::ItemOpenFolder => "Ouvrir un dossier",
        Msg::ItemSave => "Enregistrer",
        Msg::ItemSaveAs => "Enregistrer sous",
        Msg::ItemNewTab => "Nouvel onglet",
        Msg::ItemOpenInNewTab => "Ouvrir dans un nouvel onglet",
        Msg::ItemCloseTab => "Fermer l'onglet",
        Msg::ItemNextTab => "Onglet suivant",
        Msg::ItemPrevTab => "Onglet précédent",
        Msg::ItemExit => "Quitter",
        Msg::ItemUndo => "Annuler",
        Msg::ItemRedo => "Rétablir",
        Msg::ItemCopy => "Copier",
        Msg::ItemCut => "Couper",
        Msg::ItemPaste => "Coller",
        Msg::ItemSelectAll => "Tout sélectionner",
        Msg::ItemPreviewCopyPlain => "Copier en texte brut",
        Msg::ItemPreviewCopyMarkdown => "Copier en Markdown",
        Msg::ItemPreviewCopyHtml => "Copier en HTML",
        Msg::ItemPreviewSelectAll => "Tout sélectionner",
        Msg::ItemPreviewCopyLinkAddress => "Copier l'adresse du lien",
        Msg::ItemToggleView => "Changer le mode d'affichage",
        Msg::ItemEditMode => "Mode édition",
        Msg::ItemVisualEditMode => "Mode édition visuelle",
        Msg::ItemSplitPreviewMode => "Mode aperçu fractionné",
        Msg::ItemReadMode => "Mode lecture",
        Msg::ItemToggleSidebar => "Afficher/masquer la barre latérale",
        Msg::ItemFiles => "Fichiers",
        Msg::ItemOutline => "Plan",
        Msg::ItemFocusMode => "Mode concentré",
        Msg::ItemTypewriterMode => "Mode machine à écrire",
        Msg::ItemCodeLineNumbers => "Numéros de ligne de code",
        Msg::ItemFind => "Rechercher",
        Msg::ItemReplace => "Remplacer",
        Msg::ItemFindNext => "Rechercher suivant",
        Msg::ItemFindPrevious => "Rechercher précédent",
        Msg::ItemCycleTheme => "Changer le thème",
        Msg::ItemLanguage => "Langue",
        Msg::ItemBold => "Gras",
        Msg::ItemItalic => "Italique",
        Msg::ItemInlineCode => "Code en ligne",
        Msg::ItemLink => "Lien",
        Msg::ItemImage => "Image",
        Msg::ItemH1 => "Titre 1",
        Msg::ItemH2 => "Titre 2",
        Msg::ItemH3 => "Titre 3",
        Msg::ItemH4 => "Titre 4",
        Msg::ItemH5 => "Titre 5",
        Msg::ItemH6 => "Titre 6",
        Msg::ItemBullets => "Liste à puces",
        Msg::ItemNumbers => "Liste numérotée",
        Msg::ItemTask => "Tâche",
        Msg::ItemQuote => "Citation",
        Msg::ItemCodeFence => "Bloc de code",
        Msg::ItemFormatTable => "Formater le tableau",
        Msg::ItemAddTableRow => "Ajouter une ligne",
        Msg::ItemDeleteTableRow => "Supprimer la ligne",
        Msg::ItemMoveRowUp => "Déplacer la ligne vers le haut",
        Msg::ItemMoveRowDown => "Déplacer la ligne vers le bas",
        Msg::ItemAddTableColumn => "Ajouter une colonne",
        Msg::ItemDeleteTableColumn => "Supprimer la colonne",
        Msg::ItemExportHtml => "Exporter en HTML",
        Msg::ItemExportPlainHtml => "Exporter en HTML simple",
        Msg::ItemExportPdf => "Exporter en PDF",
        Msg::ItemExportLatex => "Exporter en LaTeX",
        Msg::ItemExportDocx => "Exporter en DOCX",
        Msg::ItemExportPng => "Exporter en PNG",
        Msg::ItemExportJpeg => "Exporter en JPEG",
        Msg::ItemPreferences => "Préférences",
        Msg::ItemResetPreferences => "Réinitialiser les préférences",
        Msg::ItemKeyboardShortcuts => "Raccourcis clavier",
        Msg::ItemAboutMarkion => "À propos de Markion",
        Msg::LabelEditor => "Éditeur",
        Msg::LabelPreview => "Aperçu",
        Msg::LabelFiles => "Fichiers",
        Msg::LabelOutline => "Plan",
        Msg::LabelTable => "Tableau",
        Msg::LabelImageAlt => "Texte alternatif",
        Msg::LabelImageDestination => "Destination",
        Msg::LabelImageTitle => "Titre",
        Msg::SearchFind => "Rechercher",
        Msg::SearchReplace => "Remplacer",
        Msg::SearchPrev => "Préc.",
        Msg::SearchNext => "Suiv.",
        Msg::SearchAll => "Tout",
        Msg::SearchLiteral => "Texte",
        Msg::SearchRegexMark => ".*",
        Msg::SearchCaseInsensitiveMark => "aa",
        Msg::SearchCaseSensitiveMark => "Aa",
        Msg::SearchProgress => "{0}/{1}",
        Msg::StatusReady => "Prêt",
        Msg::StatusRecoveryAvailable => "Récupération disponible",
        Msg::StatusRecoveredDocument => "Document non enregistré récupéré",
        Msg::StatusRecoveryDiscarded => "Récupération abandonnée",
        Msg::StatusWaitingFileTreeConfirm => "En attente de confirmation de l'arborescence...",
        Msg::StatusOpenCanceled => "Ouverture annulée",
        Msg::StatusOpenFolderCanceled => "Ouverture du dossier annulée",
        Msg::StatusJumpedToHeading => "Aller au titre",
        Msg::StatusNewDocument => "Nouveau document",
        Msg::StatusOpening => "Ouverture...",
        Msg::StatusOpeningFolder => "Ouverture du dossier...",
        Msg::StatusChoosingSaveLocation => "Choix de l'emplacement d'enregistrement...",
        Msg::StatusEditMode => "Mode édition",
        Msg::StatusVisualEditMode => "Mode édition visuelle",
        Msg::StatusSplitPreviewMode => "Mode aperçu fractionné",
        Msg::StatusReadMode => "Mode lecture",
        Msg::StatusSidebarShown => "Barre latérale affichée",
        Msg::StatusSidebarHidden => "Barre latérale masquée",
        Msg::StatusOutlineShown => "Plan affiché",
        Msg::StatusFileTreeShown => "Arborescence affichée",
        Msg::StatusFilteringFiles => "Filtrage des fichiers",
        Msg::StatusFileTreeRefreshed => "Arborescence actualisée",
        Msg::StatusSelectTreeEntryFirst => "Sélectionnez d'abord une entrée dans l'arborescence",
        Msg::StatusSaveBeforeRename => "Enregistrez le document actif avant de le renommer",
        Msg::StatusWaitingDeleteConfirm => "En attente de confirmation de suppression...",
        Msg::StatusDeleteCanceled => "Suppression annulée",
        Msg::StatusAboutMarkion => "À propos de Markion",
        Msg::StatusKeyboardShortcuts => "Raccourcis clavier",
        Msg::StatusUndo => "Annuler",
        Msg::StatusNothingToUndo => "Rien à annuler",
        Msg::StatusRedo => "Rétablir",
        Msg::StatusNothingToRedo => "Rien à rétablir",
        Msg::StatusNoFormattingChange => "Aucun changement de formatage",
        Msg::StatusNoTableAtCursor => "Aucun tableau à la position du curseur",
        Msg::StatusTableAlreadyFormatted => "Tableau déjà formaté",
        Msg::StatusWaitingConfirm => "En attente de confirmation...",
        Msg::StatusCanceled => "Annulé",
        Msg::StatusExitingMarkion => "Fermeture de Markion",
        Msg::StatusWaitingExitConfirm => "En attente de confirmation de fermeture...",
        Msg::StatusExitCanceled => "Fermeture annulée",
        Msg::StatusWaitingQuitConfirm => "En attente de confirmation pour quitter...",
        Msg::StatusEditingFindQuery => "Modification de la requête de recherche",
        Msg::StatusEditingReplacement => "Modification du texte de remplacement",
        Msg::StatusEditing => "Édition",
        Msg::StatusIndentedSelection => "Sélection indentée",
        Msg::StatusOutdentedSelection => "Sélection désindentée",
        Msg::StatusNothingToIndent => "Rien à indenter",
        Msg::StatusNothingToOutdent => "Rien à désindenter",
        Msg::StatusNoEdit => "Pas de modification",
        Msg::StatusClipboardEmpty => "Le presse-papier est vide",
        Msg::StatusCopiedSelection => "Sélection copiée",
        Msg::StatusCopiedPreviewPlain => "Sélection copiée en texte brut",
        Msg::StatusCopiedPreviewMarkdown => "Sélection copiée en Markdown",
        Msg::StatusCopiedPreviewHtml => "Sélection copiée en HTML",
        Msg::StatusCopiedLinkAddress => "Adresse du lien copiée",
        Msg::StatusPreviewSelectedAll => "Tout le texte de l'aperçu sélectionné",
        Msg::StatusNothingToCopy => "Rien à copier",
        Msg::StatusCutSelection => "Sélection coupée",
        Msg::StatusNothingToCut => "Rien à couper",
        Msg::StatusComposing => "Saisie en cours",
        Msg::StatusSelectedTreeEntry => "Entrée de l'arborescence sélectionnée",
        Msg::StatusNoMatchSelected => "Aucune correspondance sélectionnée",
        Msg::StatusReplacedCurrent => "Correspondance actuelle remplacée",
        Msg::StatusNoMatchesToReplace => "Aucune correspondance à remplacer",
        Msg::StatusFindQueryEmpty => "La requête de recherche est vide",
        Msg::StatusNoMatches => "Aucune correspondance",
        Msg::StatusFmtBold => "Gras",
        Msg::StatusFmtItalic => "Italique",
        Msg::StatusFmtInlineCode => "Code en ligne",
        Msg::StatusFmtLink => "Insérer un lien",
        Msg::StatusFmtImage => "Insérer une image",
        Msg::StatusFmtHeading => "Titre {0}",
        Msg::StatusFmtBulletedList => "Liste à puces",
        Msg::StatusFmtNumberedList => "Liste numérotée",
        Msg::StatusFmtTaskList => "Liste de tâches",
        Msg::StatusFmtBlockQuote => "Citation",
        Msg::StatusFmtCodeBlock => "Bloc de code",
        Msg::StatusFmtFormatTable => "Formater le tableau",
        Msg::StatusFmtAddRow => "Ajouter une ligne de tableau",
        Msg::StatusFmtDeleteRow => "Supprimer une ligne de tableau",
        Msg::StatusFmtMoveRowUp => "Déplacer la ligne vers le haut",
        Msg::StatusFmtMoveRowDown => "Déplacer la ligne vers le bas",
        Msg::StatusFmtAddColumn => "Ajouter une colonne de tableau",
        Msg::StatusFmtDeleteColumn => "Supprimer une colonne de tableau",
        Msg::StatusFocusModeOn => "Mode concentré activé",
        Msg::StatusFocusModeOff => "Mode concentré désactivé",
        Msg::StatusTypewriterModeOn => "Mode machine à écrire activé",
        Msg::StatusTypewriterModeOff => "Mode machine à écrire désactivé",
        Msg::StatusCodeLineNumbersOn => "Numéros de ligne activés",
        Msg::StatusCodeLineNumbersOff => "Numéros de ligne désactivés",
        Msg::StatusRegexFind => "Recherche par expression régulière",
        Msg::StatusLiteralFind => "Recherche littérale",
        Msg::StatusPreferences => "Préférences",
        Msg::StatusWaitingPreferenceResetConfirm => {
            "En attente de confirmation de réinitialisation..."
        }
        Msg::StatusPreferencesReset => "Préférences réinitialisées",
        Msg::StatusPreviewAdaptiveWidthOn => "Largeur adaptative de l'aperçu activée",
        Msg::StatusPreviewAdaptiveWidthOff => "Largeur adaptative de l'aperçu désactivée",
        Msg::StatusSyncScrollOn => "Défilement synchronisé activé",
        Msg::StatusSyncScrollOff => "Défilement synchronisé désactivé",
        Msg::StatusPreferenceResetCanceled => "Réinitialisation des préférences annulée",
        Msg::StatusLanguageSet => "Langue définie",
        Msg::StatusRecoveryFailed => "Échec de la récupération : {0}",
        Msg::StatusOpened => "{0} ouvert",
        Msg::StatusOpenFailed => "Échec de l'ouverture : {0}",
        Msg::StatusOpenedFolder => "Dossier ouvert : {0}",
        Msg::StatusOpenFolderFailed => "Échec de l'ouverture du dossier : {0}",
        Msg::StatusAutoSaved => "{0} enregistré automatiquement",
        Msg::StatusRecoverySaved => "{0} sauvegardé pour récupération",
        Msg::StatusAutoSaveFailed => "Échec de l'enregistrement automatique : {0}",
        Msg::StatusFindFailed => "Échec de la recherche : {0}",
        Msg::StatusPreferencesSaveFailed => "Échec de l'enregistrement des préférences : {0}",
        Msg::StatusSampleThemeSaveFailed => "Échec de l'enregistrement du thème d'exemple : {0}",
        Msg::StatusSaved => "{0} enregistré",
        Msg::StatusSaveFailed => "Échec de l'enregistrement : {0}",
        Msg::StatusSaveCanceled => "Enregistrement annulé",
        Msg::StatusExportCanceled => "Exportation annulée",
        Msg::StatusChoosingExportLocation => "Choix de l'emplacement d'export .{0}...",
        Msg::StatusExported => "{0} exporté",
        Msg::StatusExportedEngine => "{0} exporté (moteur pandoc)",
        Msg::StatusExportedBuiltin => {
            "{0} exporté (convertisseur intégré — installez pandoc pour un meilleur rendu)"
        }
        Msg::StatusExportFailed => "Échec de l'exportation : {0}",
        Msg::StatusCreated => "{0} créé",
        Msg::StatusCreateFileFailed => "Échec de la création du fichier : {0}",
        Msg::StatusCreateFolderFailed => "Échec de la création du dossier : {0}",
        Msg::StatusRenamedTo => "Renommé en {0}",
        Msg::StatusRenameFailed => "Échec du renommage : {0}",
        Msg::StatusDeleted => "{0} supprimé",
        Msg::StatusDeleteFailed => "Échec de la suppression : {0}",
        Msg::StatusShownInFileManager => "Affiché dans le gestionnaire de fichiers : {0}",
        Msg::StatusShowInFileManagerFailed => {
            "Échec de l'affichage dans le gestionnaire de fichiers : {0}"
        }
        Msg::StatusTheme => "Thème : {0}",
        Msg::StatusMatches => "{0} correspondances",
        Msg::StatusReplaceFailed => "Échec du remplacement : {0}",
        Msg::StatusReplacedMatches => "{0} correspondances remplacées",
        Msg::StatusMatchPosition => "Correspondance {0} sur {1} à {2}:{3}",
        Msg::StatusFilesVisible => "{0} fichiers visibles",
        Msg::StatusFileMatches => "{0} correspondances de fichiers",
        Msg::DialogButtonOk => "OK",
        Msg::DialogButtonCancel => "Annuler",
        Msg::DialogButtonDiscard => "Abandonner",
        Msg::DialogButtonDelete => "Supprimer",
        Msg::DialogButtonReset => "Réinitialiser",
        Msg::DialogButtonRestore => "Restaurer",
        Msg::DialogButtonExitWithoutSaving => "Quitter sans enregistrer",
        Msg::DialogAboutTitle => "À propos de Markion",
        Msg::DialogAboutDetail => {
            "Version : {0}\n\nUn éditeur Markdown local-first construit avec Rust et GPUI.\n\nGitHub : {1}"
        }
        Msg::DialogShortcutsTitle => "Raccourcis clavier",
        Msg::DialogPreferencesTitle => "Préférences",
        Msg::DialogPreferencesDetail => PREFERENCES_DETAIL_FR,
        Msg::DialogRestoreTitle => "Restaurer le document non enregistré ?",
        Msg::DialogRestoreDetail => {
            "Markion a trouvé un fichier de récupération non enregistré :\n{0}"
        }
        Msg::DialogDiscardTitle => "Abandonner les modifications non enregistrées ?",
        Msg::DialogDiscardNewDetail => {
            "Créer un nouveau document sans enregistrer les modifications actuelles."
        }
        Msg::DialogDiscardOpenDetail => {
            "Ouvrir un autre document sans enregistrer les modifications actuelles."
        }
        Msg::DialogDiscardOpenTreeDetail => {
            "Ouvrir {0} sans enregistrer les modifications actuelles."
        }
        Msg::DialogExitTitle => "Quitter Markion sans enregistrer ?",
        Msg::DialogExitDetail => "Les modifications non enregistrées seront perdues.",
        Msg::DialogDeleteTitle => "Supprimer l'entrée sélectionnée dans l'arborescence ?",
        Msg::DialogDeleteDetail => "Supprimer {0} du disque.",
        Msg::DialogDeleteFolderRecursiveTitle => "Supprimer le dossier et tout son contenu ?",
        Msg::DialogDeleteFolderRecursiveDetail => {
            "Le dossier {0} et tout son contenu seront définitivement supprimés."
        }
        Msg::DialogResetTitle => "Réinitialiser les préférences ?",
        Msg::DialogResetDetail => {
            "Les paramètres de thème, mode concentré, mode machine à écrire, numéros de ligne, largeur d'aperçu et barre latérale seront réinitialisés."
        }
        Msg::PromptOpenMarkdown => "Ouvrir Markdown",
        Msg::PromptOpenFolder => "Ouvrir un dossier",
        Msg::FileTypeMarkdown => "Document Markdown",
        Msg::FileTypeStyledHtml => "Document HTML",
        Msg::FileTypePlainHtml => "Document HTML simple",
        Msg::FileTypePdf => "Document PDF",
        Msg::FileTypeLatex => "Document LaTeX",
        Msg::FileTypeDocx => "Document Word (DOCX)",
        Msg::FileTypePng => "Image PNG",
        Msg::FileTypeJpeg => "Image JPEG",
        Msg::SummaryFilesUnit => "fichiers",
        Msg::SummaryMatchesUnit => "corresp.",
        Msg::FileTreeFilterPlaceholder => "Filtrer les fichiers",
        Msg::FileTreeFilterActive => "Filtre : {0}",
        Msg::FileTreeWorkspaceFallback => "Espace de travail",
        Msg::FileTreeMoreHidden => "... {0} de plus (filtrer pour réduire)",
        Msg::FileTreeEmptyState => "Ouvrez un fichier Markdown pour le voir ici.",
        Msg::FileTreeContextOpen => "Ouvrir",
        Msg::FileTreeContextOpenInNewTab => "Ouvrir dans un nouvel onglet",
        Msg::FileTreeContextCreateFile => "Nouveau fichier",
        Msg::FileTreeContextCreateFolder => "Nouveau dossier",
        Msg::FileTreeContextRename => "Renommer",
        Msg::FileTreeContextDelete => "Supprimer",
        Msg::FileTreeContextShowInFileManager => "Afficher dans le gestionnaire de fichiers",
        Msg::FileTreeContextRefresh => "Actualiser",
        Msg::FileTreeContextFilterFiles => "Filtrer les fichiers",
        Msg::FileTreeNamePromptLabel => "Nom",
        Msg::StatusNamingEntry => "Saisissez un nom et appuyez sur Entrée (Échap pour annuler)",
        Msg::StatusNameRequired => "Le nom ne peut pas être vide",
        Msg::PrefOn => "activé",
        Msg::PrefOff => "désactivé",
        Msg::PrefSidebarHidden => "masquée",
        Msg::CustomThemeLabel => "{0} (personnalisé)",
        Msg::PrefPanelTitle => "Préférences",
        Msg::PrefPanelThemeSection => "Thème",
        Msg::PrefPanelLanguageSection => "Langue",
        Msg::PrefPanelOtherSection => "Autre",
        Msg::PrefPanelFocusMode => "Mode concentré",
        Msg::PrefPanelTypewriterMode => "Mode machine à écrire",
        Msg::PrefPanelCodeLineNumbers => "Numéros de ligne de code",
        Msg::PrefPanelPreviewAdaptiveWidth => "Largeur adaptative de l'aperçu",
        Msg::PrefPanelSidebar => "Barre latérale",
        Msg::PrefPanelSyncScroll => "Défilement synchronisé",
        Msg::PrefPanelHeadingMenu => "Menu des titres",
        Msg::PrefPanelHeadingMenuThree => "H1–H5",
        Msg::PrefPanelHeadingMenuSix => "H1–H6",
        Msg::PrefPanelClose => "Fermer",
        Msg::DiagramLoading => "Rendu du diagramme…",
        Msg::DiagramUnsupported => "Ce format de diagramme n’est pas pris en charge.",
        Msg::DiagramInputTooLarge => "La source du diagramme dépasse la taille autorisée.",
        Msg::DiagramInvalidSource => "La source du diagramme est invalide.",
        Msg::DiagramUnsafeOutput => "La sortie du diagramme a été bloquée par sécurité.",
        Msg::DiagramRenderFailed => "Le rendu du diagramme a échoué.",
        Msg::MathRendering => "Rendu de la formule…",
        Msg::MathInvalid => "La formule est invalide.",
        Msg::MathTooLarge => "La formule dépasse la limite de rendu.",
        Msg::MathUnsupported => {
            "La formule utilise une notation ou des glyphes non pris en charge."
        }
        Msg::MathRenderFailed => "Le rendu de la formule a échoué.",
        Msg::TitleModified => "Modifié",
        Msg::TitleSaved => "Enregistré",
    }
}

const PREFERENCES_DETAIL_FR: &str = "Thème : {0}\nMode concentré : {1}\nMode machine à écrire : {2}\nNuméros de ligne : {3}\nLargeur adaptative : {4}\nBarre latérale : {5}\n\nPréférences : {6}\nThèmes personnalisés : {7}\nThèmes installés : {8}";

// ---------------------------------------------------------------------------
// German
// ---------------------------------------------------------------------------

fn de(msg: Msg) -> &'static str {
    match msg {
        Msg::MenuFile => "Datei",
        Msg::MenuEdit => "Bearbeiten",
        Msg::MenuView => "Ansicht",
        Msg::MenuFormat => "Format",
        Msg::MenuExport => "Exportieren",
        Msg::MenuHelp => "Hilfe",
        Msg::ItemNew => "Neu",
        Msg::ItemOpen => "Öffnen",
        Msg::ItemOpenFolder => "Ordner öffnen",
        Msg::ItemSave => "Speichern",
        Msg::ItemSaveAs => "Speichern unter",
        Msg::ItemNewTab => "Neuer Tab",
        Msg::ItemOpenInNewTab => "In neuem Tab öffnen",
        Msg::ItemCloseTab => "Tab schließen",
        Msg::ItemNextTab => "Nächster Tab",
        Msg::ItemPrevTab => "Vorheriger Tab",
        Msg::ItemExit => "Beenden",
        Msg::ItemUndo => "Rückgängig",
        Msg::ItemRedo => "Wiederholen",
        Msg::ItemCopy => "Kopieren",
        Msg::ItemCut => "Ausschneiden",
        Msg::ItemPaste => "Einfügen",
        Msg::ItemSelectAll => "Alles auswählen",
        Msg::ItemPreviewCopyPlain => "Als Klartext kopieren",
        Msg::ItemPreviewCopyMarkdown => "Als Markdown kopieren",
        Msg::ItemPreviewCopyHtml => "Als HTML kopieren",
        Msg::ItemPreviewSelectAll => "Alles auswählen",
        Msg::ItemPreviewCopyLinkAddress => "Linkadresse kopieren",
        Msg::ItemToggleView => "Ansichtsmodus wechseln",
        Msg::ItemEditMode => "Bearbeitungsmodus",
        Msg::ItemVisualEditMode => "Visueller Bearbeitungsmodus",
        Msg::ItemSplitPreviewMode => "Geteilter Vorschaumodus",
        Msg::ItemReadMode => "Lesemodus",
        Msg::ItemToggleSidebar => "Seitenleiste umschalten",
        Msg::ItemFiles => "Dateien",
        Msg::ItemOutline => "Gliederung",
        Msg::ItemFocusMode => "Fokusmodus",
        Msg::ItemTypewriterMode => "Schreibmaschinenmodus",
        Msg::ItemCodeLineNumbers => "Code-Zeilennummern",
        Msg::ItemFind => "Suchen",
        Msg::ItemReplace => "Ersetzen",
        Msg::ItemFindNext => "Weitersuchen",
        Msg::ItemFindPrevious => "Rückwärtssuchen",
        Msg::ItemCycleTheme => "Design wechseln",
        Msg::ItemLanguage => "Sprache",
        Msg::ItemBold => "Fett",
        Msg::ItemItalic => "Kursiv",
        Msg::ItemInlineCode => "Inline-Code",
        Msg::ItemLink => "Link",
        Msg::ItemImage => "Bild",
        Msg::ItemH1 => "Ü1",
        Msg::ItemH2 => "Ü2",
        Msg::ItemH3 => "Ü3",
        Msg::ItemH4 => "Ü4",
        Msg::ItemH5 => "Ü5",
        Msg::ItemH6 => "Ü6",
        Msg::ItemBullets => "Aufzählung",
        Msg::ItemNumbers => "Nummerierung",
        Msg::ItemTask => "Aufgabe",
        Msg::ItemQuote => "Zitat",
        Msg::ItemCodeFence => "Codeblock",
        Msg::ItemFormatTable => "Tabelle formatieren",
        Msg::ItemAddTableRow => "Zeile hinzufügen",
        Msg::ItemDeleteTableRow => "Zeile löschen",
        Msg::ItemMoveRowUp => "Zeile nach oben",
        Msg::ItemMoveRowDown => "Zeile nach unten",
        Msg::ItemAddTableColumn => "Spalte hinzufügen",
        Msg::ItemDeleteTableColumn => "Spalte löschen",
        Msg::ItemExportHtml => "HTML exportieren",
        Msg::ItemExportPlainHtml => "Einfaches HTML exportieren",
        Msg::ItemExportPdf => "PDF exportieren",
        Msg::ItemExportLatex => "LaTeX exportieren",
        Msg::ItemExportDocx => "DOCX exportieren",
        Msg::ItemExportPng => "PNG exportieren",
        Msg::ItemExportJpeg => "JPEG exportieren",
        Msg::ItemPreferences => "Einstellungen",
        Msg::ItemResetPreferences => "Einstellungen zurücksetzen",
        Msg::ItemKeyboardShortcuts => "Tastenkürzel",
        Msg::ItemAboutMarkion => "Über Markion",
        Msg::LabelEditor => "Editor",
        Msg::LabelPreview => "Vorschau",
        Msg::LabelFiles => "Dateien",
        Msg::LabelOutline => "Gliederung",
        Msg::LabelTable => "Tabelle",
        Msg::LabelImageAlt => "Alternativtext",
        Msg::LabelImageDestination => "Ziel",
        Msg::LabelImageTitle => "Titel",
        Msg::SearchFind => "Suchen",
        Msg::SearchReplace => "Ersetzen",
        Msg::SearchPrev => "Zurück",
        Msg::SearchNext => "Weiter",
        Msg::SearchAll => "Alle",
        Msg::SearchLiteral => "Text",
        Msg::SearchRegexMark => ".*",
        Msg::SearchCaseInsensitiveMark => "aa",
        Msg::SearchCaseSensitiveMark => "Aa",
        Msg::SearchProgress => "{0}/{1}",
        Msg::StatusReady => "Bereit",
        Msg::StatusRecoveryAvailable => "Wiederherstellung verfügbar",
        Msg::StatusRecoveredDocument => "Ungespeichertes Dokument wiederhergestellt",
        Msg::StatusRecoveryDiscarded => "Wiederherstellung verworfen",
        Msg::StatusWaitingFileTreeConfirm => "Warte auf Bestätigung der Dateiansicht...",
        Msg::StatusOpenCanceled => "Öffnen abgebrochen",
        Msg::StatusOpenFolderCanceled => "Ordner öffnen abgebrochen",
        Msg::StatusJumpedToHeading => "Zur Überschrift gesprungen",
        Msg::StatusNewDocument => "Neues Dokument",
        Msg::StatusOpening => "Wird geöffnet...",
        Msg::StatusOpeningFolder => "Ordner wird geöffnet...",
        Msg::StatusChoosingSaveLocation => "Speicherort wählen...",
        Msg::StatusEditMode => "Bearbeitungsmodus",
        Msg::StatusVisualEditMode => "Visueller Bearbeitungsmodus",
        Msg::StatusSplitPreviewMode => "Geteilter Vorschaumodus",
        Msg::StatusReadMode => "Lesemodus",
        Msg::StatusSidebarShown => "Seitenleiste eingeblendet",
        Msg::StatusSidebarHidden => "Seitenleiste ausgeblendet",
        Msg::StatusOutlineShown => "Gliederung eingeblendet",
        Msg::StatusFileTreeShown => "Dateiansicht eingeblendet",
        Msg::StatusFilteringFiles => "Dateien werden gefiltert",
        Msg::StatusFileTreeRefreshed => "Dateiansicht aktualisiert",
        Msg::StatusSelectTreeEntryFirst => "Zuerst einen Eintrag in der Dateiansicht auswählen",
        Msg::StatusSaveBeforeRename => "Aktives Dokument vor dem Umbenennen speichern",
        Msg::StatusWaitingDeleteConfirm => "Warte auf Löschbestätigung...",
        Msg::StatusDeleteCanceled => "Löschen abgebrochen",
        Msg::StatusAboutMarkion => "Über Markion",
        Msg::StatusKeyboardShortcuts => "Tastenkürzel",
        Msg::StatusUndo => "Rückgängig",
        Msg::StatusNothingToUndo => "Nichts rückgängig zu machen",
        Msg::StatusRedo => "Wiederholen",
        Msg::StatusNothingToRedo => "Nichts zu wiederholen",
        Msg::StatusNoFormattingChange => "Keine Formatierungsänderung",
        Msg::StatusNoTableAtCursor => "Keine Tabelle am Cursor",
        Msg::StatusTableAlreadyFormatted => "Tabelle bereits formatiert",
        Msg::StatusWaitingConfirm => "Warte auf Bestätigung...",
        Msg::StatusCanceled => "Abgebrochen",
        Msg::StatusExitingMarkion => "Markion wird beendet",
        Msg::StatusWaitingExitConfirm => "Warte auf Bestätigung zum Beenden...",
        Msg::StatusExitCanceled => "Beenden abgebrochen",
        Msg::StatusWaitingQuitConfirm => "Warte auf Bestätigung zum Beenden...",
        Msg::StatusEditingFindQuery => "Suchanfrage wird bearbeitet",
        Msg::StatusEditingReplacement => "Ersetzungstext wird bearbeitet",
        Msg::StatusEditing => "Bearbeitung",
        Msg::StatusIndentedSelection => "Auswahl eingerückt",
        Msg::StatusOutdentedSelection => "Einrückung der Auswahl aufgehoben",
        Msg::StatusNothingToIndent => "Nichts einzurücken",
        Msg::StatusNothingToOutdent => "Nichts auszurücken",
        Msg::StatusNoEdit => "Keine Bearbeitung",
        Msg::StatusClipboardEmpty => "Zwischenablage ist leer",
        Msg::StatusCopiedSelection => "Auswahl kopiert",
        Msg::StatusCopiedPreviewPlain => "Auswahl als Klartext kopiert",
        Msg::StatusCopiedPreviewMarkdown => "Auswahl als Markdown kopiert",
        Msg::StatusCopiedPreviewHtml => "Auswahl als HTML kopiert",
        Msg::StatusCopiedLinkAddress => "Linkadresse kopiert",
        Msg::StatusPreviewSelectedAll => "Gesamten Vorschautext ausgewählt",
        Msg::StatusNothingToCopy => "Nichts zum Kopieren ausgewählt",
        Msg::StatusCutSelection => "Auswahl ausgeschnitten",
        Msg::StatusNothingToCut => "Nichts zum Ausschneiden ausgewählt",
        Msg::StatusComposing => "Wird eingegeben",
        Msg::StatusSelectedTreeEntry => "Dateiansicht-Eintrag ausgewählt",
        Msg::StatusNoMatchSelected => "Kein Treffer ausgewählt",
        Msg::StatusReplacedCurrent => "Aktuellen Treffer ersetzt",
        Msg::StatusNoMatchesToReplace => "Keine Treffer zum Ersetzen",
        Msg::StatusFindQueryEmpty => "Suchanfrage ist leer",
        Msg::StatusNoMatches => "Keine Treffer",
        Msg::StatusFmtBold => "Fett",
        Msg::StatusFmtItalic => "Kursiv",
        Msg::StatusFmtInlineCode => "Inline-Code",
        Msg::StatusFmtLink => "Link einfügen",
        Msg::StatusFmtImage => "Bild einfügen",
        Msg::StatusFmtHeading => "Überschrift {0}",
        Msg::StatusFmtBulletedList => "Aufzählungsliste",
        Msg::StatusFmtNumberedList => "Nummerierte Liste",
        Msg::StatusFmtTaskList => "Aufgabenliste",
        Msg::StatusFmtBlockQuote => "Zitatblock",
        Msg::StatusFmtCodeBlock => "Codeblock",
        Msg::StatusFmtFormatTable => "Tabelle formatieren",
        Msg::StatusFmtAddRow => "Tabellenzeile hinzufügen",
        Msg::StatusFmtDeleteRow => "Tabellenzeile löschen",
        Msg::StatusFmtMoveRowUp => "Tabellenzeile nach oben",
        Msg::StatusFmtMoveRowDown => "Tabellenzeile nach unten",
        Msg::StatusFmtAddColumn => "Tabellenspalte hinzufügen",
        Msg::StatusFmtDeleteColumn => "Tabellenspalte löschen",
        Msg::StatusFocusModeOn => "Fokusmodus ein",
        Msg::StatusFocusModeOff => "Fokusmodus aus",
        Msg::StatusTypewriterModeOn => "Schreibmaschinenmodus ein",
        Msg::StatusTypewriterModeOff => "Schreibmaschinenmodus aus",
        Msg::StatusCodeLineNumbersOn => "Code-Zeilennummern ein",
        Msg::StatusCodeLineNumbersOff => "Code-Zeilennummern aus",
        Msg::StatusRegexFind => "Regex-Suche",
        Msg::StatusLiteralFind => "Textsuche",
        Msg::StatusPreferences => "Einstellungen",
        Msg::StatusWaitingPreferenceResetConfirm => "Warte auf Bestätigung zum Zurücksetzen...",
        Msg::StatusPreferencesReset => "Einstellungen zurückgesetzt",
        Msg::StatusPreviewAdaptiveWidthOn => "Adaptive Vorschaubreite ein",
        Msg::StatusPreviewAdaptiveWidthOff => "Adaptive Vorschaubreite aus",
        Msg::StatusSyncScrollOn => "Synchrones Scrollen ein",
        Msg::StatusSyncScrollOff => "Synchrones Scrollen aus",
        Msg::StatusPreferenceResetCanceled => "Zurücksetzen der Einstellungen abgebrochen",
        Msg::StatusLanguageSet => "Sprache gesetzt",
        Msg::StatusRecoveryFailed => "Wiederherstellung fehlgeschlagen: {0}",
        Msg::StatusOpened => "{0} geöffnet",
        Msg::StatusOpenFailed => "Öffnen fehlgeschlagen: {0}",
        Msg::StatusOpenedFolder => "Ordner geöffnet: {0}",
        Msg::StatusOpenFolderFailed => "Ordner öffnen fehlgeschlagen: {0}",
        Msg::StatusAutoSaved => "{0} automatisch gespeichert",
        Msg::StatusRecoverySaved => "{0} für Wiederherstellung gespeichert",
        Msg::StatusAutoSaveFailed => "Automatisches Speichern fehlgeschlagen: {0}",
        Msg::StatusFindFailed => "Suche fehlgeschlagen: {0}",
        Msg::StatusPreferencesSaveFailed => "Speichern der Einstellungen fehlgeschlagen: {0}",
        Msg::StatusSampleThemeSaveFailed => "Speichern des Beispiel-Designs fehlgeschlagen: {0}",
        Msg::StatusSaved => "{0} gespeichert",
        Msg::StatusSaveFailed => "Speichern fehlgeschlagen: {0}",
        Msg::StatusSaveCanceled => "Speichern abgebrochen",
        Msg::StatusExportCanceled => "Export abgebrochen",
        Msg::StatusChoosingExportLocation => "Exportziel für .{0} wählen...",
        Msg::StatusExported => "{0} exportiert",
        Msg::StatusExportedEngine => "{0} exportiert (pandoc-Engine)",
        Msg::StatusExportedBuiltin => {
            "{0} exportiert (integrierter Konverter — installiere pandoc für bessere Ausgabe)"
        }
        Msg::StatusExportFailed => "Export fehlgeschlagen: {0}",
        Msg::StatusCreated => "{0} erstellt",
        Msg::StatusCreateFileFailed => "Dateierstellung fehlgeschlagen: {0}",
        Msg::StatusCreateFolderFailed => "Ordnererstellung fehlgeschlagen: {0}",
        Msg::StatusRenamedTo => "Umbenannt in {0}",
        Msg::StatusRenameFailed => "Umbenennen fehlgeschlagen: {0}",
        Msg::StatusDeleted => "{0} gelöscht",
        Msg::StatusDeleteFailed => "Löschen fehlgeschlagen: {0}",
        Msg::StatusShownInFileManager => "Im Dateimanager angezeigt: {0}",
        Msg::StatusShowInFileManagerFailed => "Anzeige im Dateimanager fehlgeschlagen: {0}",
        Msg::StatusTheme => "Design: {0}",
        Msg::StatusMatches => "{0} Treffer",
        Msg::StatusReplaceFailed => "Ersetzen fehlgeschlagen: {0}",
        Msg::StatusReplacedMatches => "{0} Treffer ersetzt",
        Msg::StatusMatchPosition => "Treffer {0} von {1} bei {2}:{3}",
        Msg::StatusFilesVisible => "{0} Dateien sichtbar",
        Msg::StatusFileMatches => "{0} Dateitreffer",
        Msg::DialogButtonOk => "OK",
        Msg::DialogButtonCancel => "Abbrechen",
        Msg::DialogButtonDiscard => "Verwerfen",
        Msg::DialogButtonDelete => "Löschen",
        Msg::DialogButtonReset => "Zurücksetzen",
        Msg::DialogButtonRestore => "Wiederherstellen",
        Msg::DialogButtonExitWithoutSaving => "Ohne Speichern beenden",
        Msg::DialogAboutTitle => "Über Markion",
        Msg::DialogAboutDetail => {
            "Version: {0}\n\nEin lokal-first Markdown-Editor, entwickelt mit Rust und GPUI.\n\nGitHub: {1}"
        }
        Msg::DialogShortcutsTitle => "Tastenkürzel",
        Msg::DialogPreferencesTitle => "Einstellungen",
        Msg::DialogPreferencesDetail => PREFERENCES_DETAIL_DE,
        Msg::DialogRestoreTitle => "Ungespeichertes Dokument wiederherstellen?",
        Msg::DialogRestoreDetail => {
            "Markion hat eine ungespeicherte Wiederherstellungsdatei gefunden:\n{0}"
        }
        Msg::DialogDiscardTitle => "Ungespeicherte Änderungen verwerfen?",
        Msg::DialogDiscardNewDetail => {
            "Ein neues Dokument erstellen, ohne die aktuellen Änderungen zu speichern."
        }
        Msg::DialogDiscardOpenDetail => {
            "Ein anderes Dokument öffnen, ohne die aktuellen Änderungen zu speichern."
        }
        Msg::DialogDiscardOpenTreeDetail => {
            "{0} öffnen, ohne die aktuellen Änderungen zu speichern."
        }
        Msg::DialogExitTitle => "Markion ohne Speichern beenden?",
        Msg::DialogExitDetail => "Ungespeicherte Änderungen gehen verloren.",
        Msg::DialogDeleteTitle => "Ausgewählten Dateiansicht-Eintrag löschen?",
        Msg::DialogDeleteDetail => "{0} vom Datenträger löschen.",
        Msg::DialogDeleteFolderRecursiveTitle => "Ordner und gesamten Inhalt löschen?",
        Msg::DialogDeleteFolderRecursiveDetail => {
            "Der Ordner {0} und sein gesamter Inhalt werden endgültig gelöscht."
        }
        Msg::DialogResetTitle => "Einstellungen zurücksetzen?",
        Msg::DialogResetDetail => {
            "Design, Fokusmodus, Schreibmaschinenmodus, Zeilennummern, Vorschaubreite und Seitenleiste werden auf die Standardwerte zurückgesetzt."
        }
        Msg::PromptOpenMarkdown => "Markdown öffnen",
        Msg::PromptOpenFolder => "Ordner öffnen",
        Msg::FileTypeMarkdown => "Markdown-Dokument",
        Msg::FileTypeStyledHtml => "HTML-Dokument",
        Msg::FileTypePlainHtml => "Einfaches HTML-Dokument",
        Msg::FileTypePdf => "PDF-Dokument",
        Msg::FileTypeLatex => "LaTeX-Dokument",
        Msg::FileTypeDocx => "Word-Dokument (DOCX)",
        Msg::FileTypePng => "PNG-Bild",
        Msg::FileTypeJpeg => "JPEG-Bild",
        Msg::SummaryFilesUnit => "Dateien",
        Msg::SummaryMatchesUnit => "Treffer",
        Msg::FileTreeFilterPlaceholder => "Dateien filtern",
        Msg::FileTreeFilterActive => "Filter: {0}",
        Msg::FileTreeWorkspaceFallback => "Arbeitsbereich",
        Msg::FileTreeMoreHidden => "... {0} weitere (Filtern zum Eingrenzen)",
        Msg::FileTreeEmptyState => "Öffne eine Markdown-Datei, um sie hier anzuzeigen.",
        Msg::FileTreeContextOpen => "Öffnen",
        Msg::FileTreeContextOpenInNewTab => "In neuem Tab öffnen",
        Msg::FileTreeContextCreateFile => "Neue Datei",
        Msg::FileTreeContextCreateFolder => "Neuer Ordner",
        Msg::FileTreeContextRename => "Umbenennen",
        Msg::FileTreeContextDelete => "Löschen",
        Msg::FileTreeContextShowInFileManager => "Im Dateimanager anzeigen",
        Msg::FileTreeContextRefresh => "Aktualisieren",
        Msg::FileTreeContextFilterFiles => "Dateien filtern",
        Msg::FileTreeNamePromptLabel => "Name",
        Msg::StatusNamingEntry => "Namen eingeben und Eingabetaste drücken (Esc zum Abbrechen)",
        Msg::StatusNameRequired => "Name darf nicht leer sein",
        Msg::PrefOn => "ein",
        Msg::PrefOff => "aus",
        Msg::PrefSidebarHidden => "ausgeblendet",
        Msg::CustomThemeLabel => "{0} (benutzerdefiniert)",
        Msg::PrefPanelTitle => "Einstellungen",
        Msg::PrefPanelThemeSection => "Design",
        Msg::PrefPanelLanguageSection => "Sprache",
        Msg::PrefPanelOtherSection => "Sonstiges",
        Msg::PrefPanelFocusMode => "Fokusmodus",
        Msg::PrefPanelTypewriterMode => "Schreibmaschinenmodus",
        Msg::PrefPanelCodeLineNumbers => "Code-Zeilennummern",
        Msg::PrefPanelPreviewAdaptiveWidth => "Adaptive Vorschaubreite",
        Msg::PrefPanelSidebar => "Seitenleiste",
        Msg::PrefPanelSyncScroll => "Synchrones Scrollen",
        Msg::PrefPanelHeadingMenu => "Überschriftenmenü",
        Msg::PrefPanelHeadingMenuThree => "H1–H5",
        Msg::PrefPanelHeadingMenuSix => "H1–H6",
        Msg::PrefPanelClose => "Schließen",
        Msg::DiagramLoading => "Diagramm wird gerendert…",
        Msg::DiagramUnsupported => "Dieses Diagrammformat wird nicht unterstützt.",
        Msg::DiagramInputTooLarge => "Der Diagrammquelltext überschreitet die Größenbegrenzung.",
        Msg::DiagramInvalidSource => "Der Diagrammquelltext ist ungültig.",
        Msg::DiagramUnsafeOutput => "Die Diagrammausgabe wurde aus Sicherheitsgründen blockiert.",
        Msg::DiagramRenderFailed => "Das Rendern des Diagramms ist fehlgeschlagen.",
        Msg::MathRendering => "Formel wird gerendert…",
        Msg::MathInvalid => "Die Formel ist ungültig.",
        Msg::MathTooLarge => "Die Formel überschreitet die Rendergrößenbegrenzung.",
        Msg::MathUnsupported => {
            "Die Formel verwendet eine nicht unterstützte Notation oder Glyphe."
        }
        Msg::MathRenderFailed => "Das Rendern der Formel ist fehlgeschlagen.",
        Msg::TitleModified => "Geändert",
        Msg::TitleSaved => "Gespeichert",
    }
}

const PREFERENCES_DETAIL_DE: &str = "Design: {0}\nFokusmodus: {1}\nSchreibmaschinenmodus: {2}\nZeilennummern: {3}\nAdaptive Vorschaubreite: {4}\nSeitenleiste: {5}\n\nEinstellungen: {6}\nBenutzerdefinierte Designs: {7}\nInstallierte Designs: {8}";

// ---------------------------------------------------------------------------
// Spanish
// ---------------------------------------------------------------------------

fn es(msg: Msg) -> &'static str {
    match msg {
        Msg::MenuFile => "Archivo",
        Msg::MenuEdit => "Edición",
        Msg::MenuView => "Ver",
        Msg::MenuFormat => "Formato",
        Msg::MenuExport => "Exportar",
        Msg::MenuHelp => "Ayuda",
        Msg::ItemNew => "Nuevo",
        Msg::ItemOpen => "Abrir",
        Msg::ItemOpenFolder => "Abrir carpeta",
        Msg::ItemSave => "Guardar",
        Msg::ItemSaveAs => "Guardar como",
        Msg::ItemNewTab => "Nueva pestaña",
        Msg::ItemOpenInNewTab => "Abrir en nueva pestaña",
        Msg::ItemCloseTab => "Cerrar pestaña",
        Msg::ItemNextTab => "Pestaña siguiente",
        Msg::ItemPrevTab => "Pestaña anterior",
        Msg::ItemExit => "Salir",
        Msg::ItemUndo => "Deshacer",
        Msg::ItemRedo => "Rehacer",
        Msg::ItemCopy => "Copiar",
        Msg::ItemCut => "Cortar",
        Msg::ItemPaste => "Pegar",
        Msg::ItemSelectAll => "Seleccionar todo",
        Msg::ItemPreviewCopyPlain => "Copiar como texto sin formato",
        Msg::ItemPreviewCopyMarkdown => "Copiar como Markdown",
        Msg::ItemPreviewCopyHtml => "Copiar como HTML",
        Msg::ItemPreviewSelectAll => "Seleccionar todo",
        Msg::ItemPreviewCopyLinkAddress => "Copiar dirección del enlace",
        Msg::ItemToggleView => "Cambiar modo de vista",
        Msg::ItemEditMode => "Modo edición",
        Msg::ItemVisualEditMode => "Modo edición visual",
        Msg::ItemSplitPreviewMode => "Modo vista previa dividida",
        Msg::ItemReadMode => "Modo lectura",
        Msg::ItemToggleSidebar => "Alternar barra lateral",
        Msg::ItemFiles => "Archivos",
        Msg::ItemOutline => "Esquema",
        Msg::ItemFocusMode => "Modo concentración",
        Msg::ItemTypewriterMode => "Modo máquina de escribir",
        Msg::ItemCodeLineNumbers => "Números de línea de código",
        Msg::ItemFind => "Buscar",
        Msg::ItemReplace => "Reemplazar",
        Msg::ItemFindNext => "Buscar siguiente",
        Msg::ItemFindPrevious => "Buscar anterior",
        Msg::ItemCycleTheme => "Cambiar tema",
        Msg::ItemLanguage => "Idioma",
        Msg::ItemBold => "Negrita",
        Msg::ItemItalic => "Cursiva",
        Msg::ItemInlineCode => "Código en línea",
        Msg::ItemLink => "Enlace",
        Msg::ItemImage => "Imagen",
        Msg::ItemH1 => "T1",
        Msg::ItemH2 => "T2",
        Msg::ItemH3 => "T3",
        Msg::ItemH4 => "T4",
        Msg::ItemH5 => "T5",
        Msg::ItemH6 => "T6",
        Msg::ItemBullets => "Viñetas",
        Msg::ItemNumbers => "Numeración",
        Msg::ItemTask => "Tarea",
        Msg::ItemQuote => "Cita",
        Msg::ItemCodeFence => "Bloque de código",
        Msg::ItemFormatTable => "Formatear tabla",
        Msg::ItemAddTableRow => "Agregar fila",
        Msg::ItemDeleteTableRow => "Eliminar fila",
        Msg::ItemMoveRowUp => "Mover fila arriba",
        Msg::ItemMoveRowDown => "Mover fila abajo",
        Msg::ItemAddTableColumn => "Agregar columna",
        Msg::ItemDeleteTableColumn => "Eliminar columna",
        Msg::ItemExportHtml => "Exportar HTML",
        Msg::ItemExportPlainHtml => "Exportar HTML simple",
        Msg::ItemExportPdf => "Exportar PDF",
        Msg::ItemExportLatex => "Exportar LaTeX",
        Msg::ItemExportDocx => "Exportar DOCX",
        Msg::ItemExportPng => "Exportar PNG",
        Msg::ItemExportJpeg => "Exportar JPEG",
        Msg::ItemPreferences => "Preferencias",
        Msg::ItemResetPreferences => "Restablecer preferencias",
        Msg::ItemKeyboardShortcuts => "Atajos de teclado",
        Msg::ItemAboutMarkion => "Acerca de Markion",
        Msg::LabelEditor => "Editor",
        Msg::LabelPreview => "Vista previa",
        Msg::LabelFiles => "Archivos",
        Msg::LabelOutline => "Esquema",
        Msg::LabelTable => "Tabla",
        Msg::LabelImageAlt => "Texto alternativo",
        Msg::LabelImageDestination => "Destino",
        Msg::LabelImageTitle => "Título",
        Msg::SearchFind => "Buscar",
        Msg::SearchReplace => "Reemplazar",
        Msg::SearchPrev => "Ant.",
        Msg::SearchNext => "Sig.",
        Msg::SearchAll => "Todo",
        Msg::SearchLiteral => "Texto",
        Msg::SearchRegexMark => ".*",
        Msg::SearchCaseInsensitiveMark => "aa",
        Msg::SearchCaseSensitiveMark => "Aa",
        Msg::SearchProgress => "{0}/{1}",
        Msg::StatusReady => "Listo",
        Msg::StatusRecoveryAvailable => "Recuperación disponible",
        Msg::StatusRecoveredDocument => "Documento no guardado recuperado",
        Msg::StatusRecoveryDiscarded => "Recuperación descartada",
        Msg::StatusWaitingFileTreeConfirm => "Esperando confirmación del árbol de archivos...",
        Msg::StatusOpenCanceled => "Apertura cancelada",
        Msg::StatusOpenFolderCanceled => "Apertura de carpeta cancelada",
        Msg::StatusJumpedToHeading => "Saltar al encabezado",
        Msg::StatusNewDocument => "Nuevo documento",
        Msg::StatusOpening => "Abriendo...",
        Msg::StatusOpeningFolder => "Abriendo carpeta...",
        Msg::StatusChoosingSaveLocation => "Eligiendo ubicación para guardar...",
        Msg::StatusEditMode => "Modo edición",
        Msg::StatusVisualEditMode => "Modo edición visual",
        Msg::StatusSplitPreviewMode => "Modo vista previa dividida",
        Msg::StatusReadMode => "Modo lectura",
        Msg::StatusSidebarShown => "Barra lateral mostrada",
        Msg::StatusSidebarHidden => "Barra lateral oculta",
        Msg::StatusOutlineShown => "Esquema mostrado",
        Msg::StatusFileTreeShown => "Árbol de archivos mostrado",
        Msg::StatusFilteringFiles => "Filtrando archivos",
        Msg::StatusFileTreeRefreshed => "Árbol de archivos actualizado",
        Msg::StatusSelectTreeEntryFirst => "Seleccione primero una entrada del árbol de archivos",
        Msg::StatusSaveBeforeRename => "Guarde el documento activo antes de renombrarlo",
        Msg::StatusWaitingDeleteConfirm => "Esperando confirmación de eliminación...",
        Msg::StatusDeleteCanceled => "Eliminación cancelada",
        Msg::StatusAboutMarkion => "Acerca de Markion",
        Msg::StatusKeyboardShortcuts => "Atajos de teclado",
        Msg::StatusUndo => "Deshacer",
        Msg::StatusNothingToUndo => "Nada que deshacer",
        Msg::StatusRedo => "Rehacer",
        Msg::StatusNothingToRedo => "Nada que rehacer",
        Msg::StatusNoFormattingChange => "Sin cambios de formato",
        Msg::StatusNoTableAtCursor => "No hay tabla en el cursor",
        Msg::StatusTableAlreadyFormatted => "Tabla ya formateada",
        Msg::StatusWaitingConfirm => "Esperando confirmación...",
        Msg::StatusCanceled => "Cancelado",
        Msg::StatusExitingMarkion => "Saliendo de Markion",
        Msg::StatusWaitingExitConfirm => "Esperando confirmación para salir...",
        Msg::StatusExitCanceled => "Salida cancelada",
        Msg::StatusWaitingQuitConfirm => "Esperando confirmación para salir...",
        Msg::StatusEditingFindQuery => "Editando consulta de búsqueda",
        Msg::StatusEditingReplacement => "Editando texto de reemplazo",
        Msg::StatusEditing => "Editando",
        Msg::StatusIndentedSelection => "Selección indentada",
        Msg::StatusOutdentedSelection => "Selección desindentada",
        Msg::StatusNothingToIndent => "Nada que indentar",
        Msg::StatusNothingToOutdent => "Nada que desindentar",
        Msg::StatusNoEdit => "Sin edición",
        Msg::StatusClipboardEmpty => "El portapapeles está vacío",
        Msg::StatusCopiedSelection => "Selección copiada",
        Msg::StatusCopiedPreviewPlain => "Selección copiada como texto sin formato",
        Msg::StatusCopiedPreviewMarkdown => "Selección copiada como Markdown",
        Msg::StatusCopiedPreviewHtml => "Selección copiada como HTML",
        Msg::StatusCopiedLinkAddress => "Dirección del enlace copiada",
        Msg::StatusPreviewSelectedAll => "Todo el texto de la vista previa seleccionado",
        Msg::StatusNothingToCopy => "Nada seleccionado para copiar",
        Msg::StatusCutSelection => "Selección cortada",
        Msg::StatusNothingToCut => "Nada seleccionado para cortar",
        Msg::StatusComposing => "Componiendo",
        Msg::StatusSelectedTreeEntry => "Entrada del árbol de archivos seleccionada",
        Msg::StatusNoMatchSelected => "Ninguna coincidencia seleccionada",
        Msg::StatusReplacedCurrent => "Coincidencia actual reemplazada",
        Msg::StatusNoMatchesToReplace => "No hay coincidencias para reemplazar",
        Msg::StatusFindQueryEmpty => "La consulta de búsqueda está vacía",
        Msg::StatusNoMatches => "Sin coincidencias",
        Msg::StatusFmtBold => "Negrita",
        Msg::StatusFmtItalic => "Cursiva",
        Msg::StatusFmtInlineCode => "Código en línea",
        Msg::StatusFmtLink => "Insertar enlace",
        Msg::StatusFmtImage => "Insertar imagen",
        Msg::StatusFmtHeading => "Encabezado {0}",
        Msg::StatusFmtBulletedList => "Lista con viñetas",
        Msg::StatusFmtNumberedList => "Lista numerada",
        Msg::StatusFmtTaskList => "Lista de tareas",
        Msg::StatusFmtBlockQuote => "Bloque de cita",
        Msg::StatusFmtCodeBlock => "Bloque de código",
        Msg::StatusFmtFormatTable => "Formatear tabla",
        Msg::StatusFmtAddRow => "Agregar fila de tabla",
        Msg::StatusFmtDeleteRow => "Eliminar fila de tabla",
        Msg::StatusFmtMoveRowUp => "Mover fila arriba",
        Msg::StatusFmtMoveRowDown => "Mover fila abajo",
        Msg::StatusFmtAddColumn => "Agregar columna de tabla",
        Msg::StatusFmtDeleteColumn => "Eliminar columna de tabla",
        Msg::StatusFocusModeOn => "Modo concentración activado",
        Msg::StatusFocusModeOff => "Modo concentración desactivado",
        Msg::StatusTypewriterModeOn => "Modo máquina de escribir activado",
        Msg::StatusTypewriterModeOff => "Modo máquina de escribir desactivado",
        Msg::StatusCodeLineNumbersOn => "Números de línea activados",
        Msg::StatusCodeLineNumbersOff => "Números de línea desactivados",
        Msg::StatusRegexFind => "Búsqueda regex",
        Msg::StatusLiteralFind => "Búsqueda literal",
        Msg::StatusPreferences => "Preferencias",
        Msg::StatusWaitingPreferenceResetConfirm => "Esperando confirmación para restablecer...",
        Msg::StatusPreferencesReset => "Preferencias restablecidas",
        Msg::StatusPreviewAdaptiveWidthOn => "Ancho adaptativo de vista previa activado",
        Msg::StatusPreviewAdaptiveWidthOff => "Ancho adaptativo de vista previa desactivado",
        Msg::StatusSyncScrollOn => "Desplazamiento sincronizado activado",
        Msg::StatusSyncScrollOff => "Desplazamiento sincronizado desactivado",
        Msg::StatusPreferenceResetCanceled => "Restablecimiento de preferencias cancelado",
        Msg::StatusLanguageSet => "Idioma establecido",
        Msg::StatusRecoveryFailed => "Error de recuperación: {0}",
        Msg::StatusOpened => "{0} abierto",
        Msg::StatusOpenFailed => "Error al abrir: {0}",
        Msg::StatusOpenedFolder => "Carpeta abierta: {0}",
        Msg::StatusOpenFolderFailed => "Error al abrir la carpeta: {0}",
        Msg::StatusAutoSaved => "{0} guardado automáticamente",
        Msg::StatusRecoverySaved => "{0} guardado para recuperación",
        Msg::StatusAutoSaveFailed => "Error de guardado automático: {0}",
        Msg::StatusFindFailed => "Error de búsqueda: {0}",
        Msg::StatusPreferencesSaveFailed => "Error al guardar preferencias: {0}",
        Msg::StatusSampleThemeSaveFailed => "Error al guardar tema de ejemplo: {0}",
        Msg::StatusSaved => "{0} guardado",
        Msg::StatusSaveFailed => "Error al guardar: {0}",
        Msg::StatusSaveCanceled => "Guardado cancelado",
        Msg::StatusExportCanceled => "Exportación cancelada",
        Msg::StatusChoosingExportLocation => "Eligiendo ubicación de exportación .{0}...",
        Msg::StatusExported => "{0} exportado",
        Msg::StatusExportedEngine => "{0} exportado (motor pandoc)",
        Msg::StatusExportedBuiltin => {
            "{0} exportado (conversor integrado — instale pandoc para una salida más rica)"
        }
        Msg::StatusExportFailed => "Error de exportación: {0}",
        Msg::StatusCreated => "{0} creado",
        Msg::StatusCreateFileFailed => "Error al crear archivo: {0}",
        Msg::StatusCreateFolderFailed => "Error al crear carpeta: {0}",
        Msg::StatusRenamedTo => "Renombrado a {0}",
        Msg::StatusRenameFailed => "Error al renombrar: {0}",
        Msg::StatusDeleted => "{0} eliminado",
        Msg::StatusDeleteFailed => "Error al eliminar: {0}",
        Msg::StatusShownInFileManager => "Mostrado en el administrador de archivos: {0}",
        Msg::StatusShowInFileManagerFailed => {
            "Error al mostrar en el administrador de archivos: {0}"
        }
        Msg::StatusTheme => "Tema: {0}",
        Msg::StatusMatches => "{0} coincidencias",
        Msg::StatusReplaceFailed => "Error de reemplazo: {0}",
        Msg::StatusReplacedMatches => "{0} coincidencias reemplazadas",
        Msg::StatusMatchPosition => "Coincidencia {0} de {1} en {2}:{3}",
        Msg::StatusFilesVisible => "{0} archivos visibles",
        Msg::StatusFileMatches => "{0} coincidencias de archivos",
        Msg::DialogButtonOk => "Aceptar",
        Msg::DialogButtonCancel => "Cancelar",
        Msg::DialogButtonDiscard => "Descartar",
        Msg::DialogButtonDelete => "Eliminar",
        Msg::DialogButtonReset => "Restablecer",
        Msg::DialogButtonRestore => "Restaurar",
        Msg::DialogButtonExitWithoutSaving => "Salir sin guardar",
        Msg::DialogAboutTitle => "Acerca de Markion",
        Msg::DialogAboutDetail => {
            "Versión: {0}\n\nUn editor Markdown local-first construido con Rust y GPUI.\n\nGitHub: {1}"
        }
        Msg::DialogShortcutsTitle => "Atajos de teclado",
        Msg::DialogPreferencesTitle => "Preferencias",
        Msg::DialogPreferencesDetail => PREFERENCES_DETAIL_ES,
        Msg::DialogRestoreTitle => "¿Restaurar documento no guardado?",
        Msg::DialogRestoreDetail => "Markion encontró un archivo de recuperación no guardado:\n{0}",
        Msg::DialogDiscardTitle => "¿Descartar cambios no guardados?",
        Msg::DialogDiscardNewDetail => "Crear un nuevo documento sin guardar los cambios actuales.",
        Msg::DialogDiscardOpenDetail => "Abrir otro documento sin guardar los cambios actuales.",
        Msg::DialogDiscardOpenTreeDetail => "Abrir {0} sin guardar los cambios actuales.",
        Msg::DialogExitTitle => "¿Salir de Markion sin guardar?",
        Msg::DialogExitDetail => "Los cambios no guardados se perderán.",
        Msg::DialogDeleteTitle => "¿Eliminar la entrada seleccionada del árbol de archivos?",
        Msg::DialogDeleteDetail => "Eliminar {0} del disco.",
        Msg::DialogDeleteFolderRecursiveTitle => "¿Eliminar carpeta y todo su contenido?",
        Msg::DialogDeleteFolderRecursiveDetail => {
            "La carpeta {0} y todo su contenido se eliminarán permanentemente."
        }
        Msg::DialogResetTitle => "¿Restablecer preferencias?",
        Msg::DialogResetDetail => {
            "Tema, modo concentración, modo máquina de escribir, números de línea, ancho de vista previa y barra lateral volverán a los valores predeterminados."
        }
        Msg::PromptOpenMarkdown => "Abrir Markdown",
        Msg::PromptOpenFolder => "Abrir carpeta",
        Msg::FileTypeMarkdown => "Documento Markdown",
        Msg::FileTypeStyledHtml => "Documento HTML",
        Msg::FileTypePlainHtml => "Documento HTML simple",
        Msg::FileTypePdf => "Documento PDF",
        Msg::FileTypeLatex => "Documento LaTeX",
        Msg::FileTypeDocx => "Documento Word (DOCX)",
        Msg::FileTypePng => "Imagen PNG",
        Msg::FileTypeJpeg => "Imagen JPEG",
        Msg::SummaryFilesUnit => "archivos",
        Msg::SummaryMatchesUnit => "coincid.",
        Msg::FileTreeFilterPlaceholder => "Filtrar archivos",
        Msg::FileTreeFilterActive => "Filtro: {0}",
        Msg::FileTreeWorkspaceFallback => "Espacio de trabajo",
        Msg::FileTreeMoreHidden => "... {0} más (filtrar para reducir)",
        Msg::FileTreeEmptyState => "Abra un archivo Markdown para verlo aquí.",
        Msg::FileTreeContextOpen => "Abrir",
        Msg::FileTreeContextOpenInNewTab => "Abrir en nueva pestaña",
        Msg::FileTreeContextCreateFile => "Nuevo archivo",
        Msg::FileTreeContextCreateFolder => "Nueva carpeta",
        Msg::FileTreeContextRename => "Renombrar",
        Msg::FileTreeContextDelete => "Eliminar",
        Msg::FileTreeContextShowInFileManager => "Mostrar en el administrador de archivos",
        Msg::FileTreeContextRefresh => "Actualizar",
        Msg::FileTreeContextFilterFiles => "Filtrar archivos",
        Msg::FileTreeNamePromptLabel => "Nombre",
        Msg::StatusNamingEntry => "Escriba un nombre y presione Enter (Esc para cancelar)",
        Msg::StatusNameRequired => "El nombre no puede estar vacío",
        Msg::PrefOn => "activado",
        Msg::PrefOff => "desactivado",
        Msg::PrefSidebarHidden => "oculta",
        Msg::CustomThemeLabel => "{0} (personalizado)",
        Msg::PrefPanelTitle => "Preferencias",
        Msg::PrefPanelThemeSection => "Tema",
        Msg::PrefPanelLanguageSection => "Idioma",
        Msg::PrefPanelOtherSection => "Otro",
        Msg::PrefPanelFocusMode => "Modo concentración",
        Msg::PrefPanelTypewriterMode => "Modo máquina de escribir",
        Msg::PrefPanelCodeLineNumbers => "Números de línea de código",
        Msg::PrefPanelPreviewAdaptiveWidth => "Ancho adaptativo de vista previa",
        Msg::PrefPanelSidebar => "Barra lateral",
        Msg::PrefPanelSyncScroll => "Desplazamiento sincronizado",
        Msg::PrefPanelHeadingMenu => "Menú de encabezados",
        Msg::PrefPanelHeadingMenuThree => "H1–H5",
        Msg::PrefPanelHeadingMenuSix => "H1–H6",
        Msg::PrefPanelClose => "Cerrar",
        Msg::DiagramLoading => "Renderizando diagrama…",
        Msg::DiagramUnsupported => "Este formato de diagrama no es compatible.",
        Msg::DiagramInputTooLarge => "El código del diagrama supera el límite de tamaño.",
        Msg::DiagramInvalidSource => "El código del diagrama no es válido.",
        Msg::DiagramUnsafeOutput => "La salida del diagrama se bloqueó por seguridad.",
        Msg::DiagramRenderFailed => "No se pudo renderizar el diagrama.",
        Msg::MathRendering => "Renderizando fórmula…",
        Msg::MathInvalid => "La fórmula no es válida.",
        Msg::MathTooLarge => "La fórmula supera el límite de tamaño de renderizado.",
        Msg::MathUnsupported => "La fórmula usa notación o glifos no compatibles.",
        Msg::MathRenderFailed => "No se pudo renderizar la fórmula.",
        Msg::TitleModified => "Modificado",
        Msg::TitleSaved => "Guardado",
    }
}

const PREFERENCES_DETAIL_ES: &str = "Tema: {0}\nModo concentración: {1}\nModo máquina de escribir: {2}\nNúmeros de línea: {3}\nAncho adaptativo: {4}\nBarra lateral: {5}\n\nPreferencias: {6}\nTemas personalizados: {7}\nTemas instalados: {8}";

// ---------------------------------------------------------------------------
// Chinese (Simplified)
// ---------------------------------------------------------------------------

fn zh(msg: Msg) -> &'static str {
    match msg {
        Msg::MenuFile => "文件",
        Msg::MenuEdit => "编辑",
        Msg::MenuView => "视图",
        Msg::MenuFormat => "格式",
        Msg::MenuExport => "导出",
        Msg::MenuHelp => "帮助",

        Msg::ItemNew => "新建",
        Msg::ItemOpen => "打开",
        Msg::ItemOpenFolder => "打开文件夹",
        Msg::ItemSave => "保存",
        Msg::ItemSaveAs => "另存为",
        Msg::ItemNewTab => "新建标签页",
        Msg::ItemOpenInNewTab => "在新标签页打开",
        Msg::ItemCloseTab => "关闭标签页",
        Msg::ItemNextTab => "下一个标签页",
        Msg::ItemPrevTab => "上一个标签页",
        Msg::ItemExit => "退出",

        Msg::ItemUndo => "撤销",
        Msg::ItemRedo => "重做",
        Msg::ItemCopy => "复制",
        Msg::ItemCut => "剪切",
        Msg::ItemPaste => "粘贴",
        Msg::ItemSelectAll => "全选",
        Msg::ItemPreviewCopyPlain => "复制为纯文本",
        Msg::ItemPreviewCopyMarkdown => "复制为 Markdown",
        Msg::ItemPreviewCopyHtml => "复制为 HTML",
        Msg::ItemPreviewSelectAll => "全选",
        Msg::ItemPreviewCopyLinkAddress => "复制链接地址",

        Msg::ItemToggleView => "循环切换视图模式",
        Msg::ItemEditMode => "编辑模式",
        Msg::ItemVisualEditMode => "可视化编辑模式",
        Msg::ItemSplitPreviewMode => "分栏预览模式",
        Msg::ItemReadMode => "阅读模式",
        Msg::ItemToggleSidebar => "切换侧边栏",
        Msg::ItemFiles => "文件",
        Msg::ItemOutline => "大纲",
        Msg::ItemFocusMode => "专注模式",
        Msg::ItemTypewriterMode => "打字机模式",
        Msg::ItemCodeLineNumbers => "代码行号",
        Msg::ItemFind => "查找",
        Msg::ItemReplace => "替换",
        Msg::ItemFindNext => "查找下一个",
        Msg::ItemFindPrevious => "查找上一个",
        Msg::ItemCycleTheme => "切换主题",
        Msg::ItemLanguage => "语言",

        Msg::ItemBold => "加粗",
        Msg::ItemItalic => "斜体",
        Msg::ItemInlineCode => "行内代码",
        Msg::ItemLink => "链接",
        Msg::ItemImage => "图片",
        Msg::ItemH1 => "一级标题",
        Msg::ItemH2 => "二级标题",
        Msg::ItemH3 => "三级标题",
        Msg::ItemH4 => "四级标题",
        Msg::ItemH5 => "五级标题",
        Msg::ItemH6 => "六级标题",
        Msg::ItemBullets => "无序列表",
        Msg::ItemNumbers => "有序列表",
        Msg::ItemTask => "任务列表",
        Msg::ItemQuote => "引用",
        Msg::ItemCodeFence => "代码块",
        Msg::ItemFormatTable => "格式化表格",
        Msg::ItemAddTableRow => "添加表格行",
        Msg::ItemDeleteTableRow => "删除表格行",
        Msg::ItemMoveRowUp => "上移表格行",
        Msg::ItemMoveRowDown => "下移表格行",
        Msg::ItemAddTableColumn => "添加表格列",
        Msg::ItemDeleteTableColumn => "删除表格列",

        Msg::ItemExportHtml => "导出 HTML",
        Msg::ItemExportPlainHtml => "导出纯 HTML",
        Msg::ItemExportPdf => "导出 PDF",
        Msg::ItemExportLatex => "导出 LaTeX",
        Msg::ItemExportDocx => "导出 DOCX",
        Msg::ItemExportPng => "导出 PNG",
        Msg::ItemExportJpeg => "导出 JPEG",

        Msg::ItemPreferences => "首选项",
        Msg::ItemResetPreferences => "重置首选项",
        Msg::ItemKeyboardShortcuts => "键盘快捷键",
        Msg::ItemAboutMarkion => "关于 Markion",

        Msg::LabelEditor => "编辑器",
        Msg::LabelPreview => "预览",
        Msg::LabelFiles => "文件",
        Msg::LabelOutline => "大纲",
        Msg::LabelTable => "表格",
        Msg::LabelImageAlt => "替代文本",
        Msg::LabelImageDestination => "目标地址",
        Msg::LabelImageTitle => "标题",

        Msg::SearchFind => "查找",
        Msg::SearchReplace => "替换",
        Msg::SearchPrev => "上一个",
        Msg::SearchNext => "下一个",
        Msg::SearchAll => "全部",
        Msg::SearchLiteral => "文本",
        Msg::SearchRegexMark => ".*",
        Msg::SearchCaseInsensitiveMark => "aa",
        Msg::SearchCaseSensitiveMark => "Aa",
        Msg::SearchProgress => "{0}/{1}",

        Msg::StatusReady => "就绪",
        Msg::StatusRecoveryAvailable => "有可恢复的文档",
        Msg::StatusRecoveredDocument => "已恢复未保存的文档",
        Msg::StatusRecoveryDiscarded => "已放弃恢复",
        Msg::StatusWaitingFileTreeConfirm => "等待文件树确认…",
        Msg::StatusOpenCanceled => "已取消打开",
        Msg::StatusOpenFolderCanceled => "已取消打开文件夹",
        Msg::StatusJumpedToHeading => "已跳转到标题",
        Msg::StatusNewDocument => "新文档",
        Msg::StatusOpening => "正在打开…",
        Msg::StatusOpeningFolder => "正在打开文件夹…",
        Msg::StatusChoosingSaveLocation => "正在选择保存位置…",
        Msg::StatusEditMode => "编辑模式",
        Msg::StatusVisualEditMode => "可视化编辑模式",
        Msg::StatusSplitPreviewMode => "分栏预览模式",
        Msg::StatusReadMode => "阅读模式",
        Msg::StatusSidebarShown => "已显示侧边栏",
        Msg::StatusSidebarHidden => "已隐藏侧边栏",
        Msg::StatusOutlineShown => "已显示大纲",
        Msg::StatusFileTreeShown => "已显示文件树",
        Msg::StatusFilteringFiles => "正在筛选文件",
        Msg::StatusFileTreeRefreshed => "已刷新文件树",
        Msg::StatusSelectTreeEntryFirst => "请先选择一个文件树条目",
        Msg::StatusSaveBeforeRename => "重命名前请先保存当前文档",
        Msg::StatusWaitingDeleteConfirm => "等待删除确认…",
        Msg::StatusDeleteCanceled => "已取消删除",
        Msg::StatusAboutMarkion => "关于 Markion",
        Msg::StatusKeyboardShortcuts => "键盘快捷键",
        Msg::StatusUndo => "已撤销",
        Msg::StatusNothingToUndo => "没有可撤销的操作",
        Msg::StatusRedo => "已重做",
        Msg::StatusNothingToRedo => "没有可重做的操作",
        Msg::StatusNoFormattingChange => "格式无变化",
        Msg::StatusNoTableAtCursor => "光标处没有表格",
        Msg::StatusTableAlreadyFormatted => "表格已是格式化的",
        Msg::StatusWaitingConfirm => "等待确认…",
        Msg::StatusCanceled => "已取消",
        Msg::StatusExitingMarkion => "正在退出 Markion",
        Msg::StatusWaitingExitConfirm => "等待退出确认…",
        Msg::StatusExitCanceled => "已取消退出",
        Msg::StatusWaitingQuitConfirm => "等待退出确认…",
        Msg::StatusEditingFindQuery => "正在编辑查找内容",
        Msg::StatusEditingReplacement => "正在编辑替换内容",
        Msg::StatusEditing => "编辑中",
        Msg::StatusIndentedSelection => "已增加缩进",
        Msg::StatusOutdentedSelection => "已减少缩进",
        Msg::StatusNothingToIndent => "没有可缩进的内容",
        Msg::StatusNothingToOutdent => "没有可减少缩进的内容",
        Msg::StatusNoEdit => "无编辑",
        Msg::StatusClipboardEmpty => "剪贴板为空",
        Msg::StatusCopiedSelection => "已复制所选内容",
        Msg::StatusCopiedPreviewPlain => "已复制为纯文本",
        Msg::StatusCopiedPreviewMarkdown => "已复制为 Markdown",
        Msg::StatusCopiedPreviewHtml => "已复制为 HTML",
        Msg::StatusCopiedLinkAddress => "已复制链接地址",
        Msg::StatusPreviewSelectedAll => "已选中全部预览文本",
        Msg::StatusNothingToCopy => "没有可复制的内容",
        Msg::StatusCutSelection => "已剪切所选内容",
        Msg::StatusNothingToCut => "没有可剪切的内容",
        Msg::StatusComposing => "输入中",
        Msg::StatusSelectedTreeEntry => "已选择文件树条目",
        Msg::StatusNoMatchSelected => "未选中匹配项",
        Msg::StatusReplacedCurrent => "已替换当前匹配",
        Msg::StatusNoMatchesToReplace => "没有可替换的匹配",
        Msg::StatusFindQueryEmpty => "查找内容为空",
        Msg::StatusNoMatches => "未找到匹配",

        Msg::StatusFmtBold => "加粗",
        Msg::StatusFmtItalic => "斜体",
        Msg::StatusFmtInlineCode => "行内代码",
        Msg::StatusFmtLink => "插入链接",
        Msg::StatusFmtImage => "插入图片",
        Msg::StatusFmtHeading => "{0} 级标题",
        Msg::StatusFmtBulletedList => "无序列表",
        Msg::StatusFmtNumberedList => "有序列表",
        Msg::StatusFmtTaskList => "任务列表",
        Msg::StatusFmtBlockQuote => "引用",
        Msg::StatusFmtCodeBlock => "代码块",
        Msg::StatusFmtFormatTable => "格式化表格",
        Msg::StatusFmtAddRow => "添加表格行",
        Msg::StatusFmtDeleteRow => "删除表格行",
        Msg::StatusFmtMoveRowUp => "上移表格行",
        Msg::StatusFmtMoveRowDown => "下移表格行",
        Msg::StatusFmtAddColumn => "添加表格列",
        Msg::StatusFmtDeleteColumn => "删除表格列",
        Msg::StatusFocusModeOn => "专注模式已开启",
        Msg::StatusFocusModeOff => "专注模式已关闭",
        Msg::StatusTypewriterModeOn => "打字机模式已开启",
        Msg::StatusTypewriterModeOff => "打字机模式已关闭",
        Msg::StatusCodeLineNumbersOn => "代码行号已开启",
        Msg::StatusCodeLineNumbersOff => "代码行号已关闭",
        Msg::StatusRegexFind => "正则查找",
        Msg::StatusLiteralFind => "字面查找",
        Msg::StatusPreferences => "首选项",
        Msg::StatusWaitingPreferenceResetConfirm => "等待重置首选项确认…",
        Msg::StatusPreferencesReset => "首选项已重置",
        Msg::StatusPreviewAdaptiveWidthOn => "预览自适应宽度已开启",
        Msg::StatusPreviewAdaptiveWidthOff => "预览自适应宽度已关闭",
        Msg::StatusSyncScrollOn => "同步滚动已开启",
        Msg::StatusSyncScrollOff => "同步滚动已关闭",
        Msg::StatusPreferenceResetCanceled => "已取消重置首选项",
        Msg::StatusLanguageSet => "已设置语言",

        Msg::StatusRecoveryFailed => "恢复失败：{0}",
        Msg::StatusOpened => "已打开 {0}",
        Msg::StatusOpenFailed => "打开失败：{0}",
        Msg::StatusOpenedFolder => "已打开文件夹 {0}",
        Msg::StatusOpenFolderFailed => "打开文件夹失败：{0}",
        Msg::StatusAutoSaved => "已自动保存 {0}",
        Msg::StatusRecoverySaved => "已保存恢复副本 {0}",
        Msg::StatusAutoSaveFailed => "自动保存失败：{0}",
        Msg::StatusFindFailed => "查找失败：{0}",
        Msg::StatusPreferencesSaveFailed => "首选项保存失败：{0}",
        Msg::StatusSampleThemeSaveFailed => "示例主题保存失败：{0}",
        Msg::StatusSaved => "已保存 {0}",
        Msg::StatusSaveFailed => "保存失败：{0}",
        Msg::StatusSaveCanceled => "已取消保存",
        Msg::StatusExportCanceled => "已取消导出",
        Msg::StatusChoosingExportLocation => "正在选择 .{0} 导出位置…",
        Msg::StatusExported => "已导出 {0}",
        Msg::StatusExportedEngine => "已导出 {0}（pandoc 引擎）",
        Msg::StatusExportedBuiltin => "已导出 {0}（内置简易导出——安装 pandoc 可获得更高质量）",
        Msg::StatusExportFailed => "导出失败：{0}",
        Msg::StatusCreated => "已创建 {0}",
        Msg::StatusCreateFileFailed => "创建文件失败：{0}",
        Msg::StatusCreateFolderFailed => "创建文件夹失败：{0}",
        Msg::StatusRenamedTo => "已重命名为 {0}",
        Msg::StatusRenameFailed => "重命名失败：{0}",
        Msg::StatusDeleted => "已删除 {0}",
        Msg::StatusDeleteFailed => "删除失败：{0}",
        Msg::StatusShownInFileManager => "已在系统资源管理器中显示：{0}",
        Msg::StatusShowInFileManagerFailed => "在系统资源管理器中显示失败：{0}",
        Msg::StatusTheme => "主题：{0}",
        Msg::StatusMatches => "找到 {0} 个匹配",
        Msg::StatusReplaceFailed => "替换失败：{0}",
        Msg::StatusReplacedMatches => "已替换 {0} 处匹配",
        Msg::StatusMatchPosition => "第 {0} 个，共 {1} 个匹配（{2}:{3}）",
        Msg::StatusFilesVisible => "可见 {0} 个文件",
        Msg::StatusFileMatches => "{0} 个匹配文件",

        Msg::DialogButtonOk => "确定",
        Msg::DialogButtonCancel => "取消",
        Msg::DialogButtonDiscard => "放弃",
        Msg::DialogButtonDelete => "删除",
        Msg::DialogButtonReset => "重置",
        Msg::DialogButtonRestore => "恢复",
        Msg::DialogButtonExitWithoutSaving => "不保存退出",

        Msg::DialogAboutTitle => "关于 Markion",
        Msg::DialogAboutDetail => {
            "版本：{0}\n\n一款使用 Rust 与 GPUI 构建的本地优先 Markdown 编辑器。\n\nGitHub：{1}"
        }
        Msg::DialogShortcutsTitle => "键盘快捷键",
        Msg::DialogPreferencesTitle => "首选项",
        Msg::DialogPreferencesDetail => PREFERENCES_DETAIL_ZH,
        Msg::DialogRestoreTitle => "恢复未保存的文档？",
        Msg::DialogRestoreDetail => "Markion 发现了一个未保存的恢复文件：\n{0}",
        Msg::DialogDiscardTitle => "放弃未保存的更改？",
        Msg::DialogDiscardNewDetail => "新建文档而不保存当前更改。",
        Msg::DialogDiscardOpenDetail => "打开其他文档而不保存当前更改。",
        Msg::DialogDiscardOpenTreeDetail => "打开 {0} 而不保存当前更改。",
        Msg::DialogExitTitle => "不保存就退出 Markion？",
        Msg::DialogExitDetail => "未保存的更改将会丢失。",
        Msg::DialogDeleteTitle => "删除选中的文件树条目？",
        Msg::DialogDeleteDetail => "从磁盘删除 {0}。",
        Msg::DialogDeleteFolderRecursiveTitle => "删除文件夹及其所有内容？",
        Msg::DialogDeleteFolderRecursiveDetail => "文件夹 {0} 及其中的所有内容将被永久删除。",
        Msg::DialogResetTitle => "重置首选项？",
        Msg::DialogResetDetail => {
            "主题、专注模式、打字机模式、代码行号、预览宽度以及侧边栏设置将恢复为默认值。"
        }
        Msg::PromptOpenMarkdown => "打开 Markdown",
        Msg::PromptOpenFolder => "打开文件夹",
        Msg::FileTypeMarkdown => "Markdown 文档",
        Msg::FileTypeStyledHtml => "HTML 文档",
        Msg::FileTypePlainHtml => "纯 HTML 文档",
        Msg::FileTypePdf => "PDF 文档",
        Msg::FileTypeLatex => "LaTeX 文档",
        Msg::FileTypeDocx => "Word 文档（DOCX）",
        Msg::FileTypePng => "PNG 图像",
        Msg::FileTypeJpeg => "JPEG 图像",

        Msg::SummaryFilesUnit => "个文件",
        Msg::SummaryMatchesUnit => "个匹配",

        Msg::FileTreeFilterPlaceholder => "筛选文件",
        Msg::FileTreeFilterActive => "筛选：{0}",
        Msg::FileTreeWorkspaceFallback => "工作区",
        Msg::FileTreeMoreHidden => "……还有 {0} 项（筛选以缩小范围）",
        Msg::FileTreeEmptyState => "打开一个 Markdown 文件即可在此处查看。",
        Msg::FileTreeContextOpen => "打开",
        Msg::FileTreeContextOpenInNewTab => "在新标签页中打开",
        Msg::FileTreeContextCreateFile => "新建文件",
        Msg::FileTreeContextCreateFolder => "新建文件夹",
        Msg::FileTreeContextRename => "重命名",
        Msg::FileTreeContextDelete => "删除",
        Msg::FileTreeContextShowInFileManager => "在系统资源管理器中显示",
        Msg::FileTreeContextRefresh => "刷新",
        Msg::FileTreeContextFilterFiles => "筛选文件",
        Msg::FileTreeNamePromptLabel => "名称",
        Msg::StatusNamingEntry => "输入名称后按回车确认（Esc 取消）",
        Msg::StatusNameRequired => "名称不能为空",

        Msg::PrefOn => "开",
        Msg::PrefOff => "关",
        Msg::PrefSidebarHidden => "隐藏",
        Msg::CustomThemeLabel => "{0}（自定义）",

        Msg::PrefPanelTitle => "首选项",
        Msg::PrefPanelThemeSection => "主题",
        Msg::PrefPanelLanguageSection => "语言",
        Msg::PrefPanelOtherSection => "其他",
        Msg::PrefPanelFocusMode => "专注模式",
        Msg::PrefPanelTypewriterMode => "打字机模式",
        Msg::PrefPanelCodeLineNumbers => "代码行号",
        Msg::PrefPanelPreviewAdaptiveWidth => "预览自适应宽度",
        Msg::PrefPanelSyncScroll => "同步滚动",
        Msg::PrefPanelSidebar => "侧边栏",
        Msg::PrefPanelHeadingMenu => "标题菜单",
        Msg::PrefPanelHeadingMenuThree => "H1–H5",
        Msg::PrefPanelHeadingMenuSix => "H1–H6",
        Msg::PrefPanelClose => "关闭",
        Msg::DiagramLoading => "正在渲染图表…",
        Msg::DiagramUnsupported => "不支持此图表格式。",
        Msg::DiagramInputTooLarge => "图表源代码超过大小限制。",
        Msg::DiagramInvalidSource => "图表源代码无效。",
        Msg::DiagramUnsafeOutput => "出于安全原因，图表输出已被拦截。",
        Msg::DiagramRenderFailed => "图表渲染失败。",
        Msg::MathRendering => "正在渲染公式…",
        Msg::MathInvalid => "公式无效。",
        Msg::MathTooLarge => "公式超过渲染大小限制。",
        Msg::MathUnsupported => "公式包含不支持的记法或字形。",
        Msg::MathRenderFailed => "公式渲染失败。",
        Msg::TitleModified => "已修改",
        Msg::TitleSaved => "已保存",
    }
}

const PREFERENCES_DETAIL_ZH: &str = "主题：{0}\n专注模式：{1}\n打字机模式：{2}\n代码行号：{3}\n预览自适应宽度：{4}\n侧边栏：{5}\n\n首选项：{6}\n主题目录：{7}\n已安装自定义主题：{8}";

// ---------------------------------------------------------------------------
// Traditional Chinese (Taiwan regional terminology)
// ---------------------------------------------------------------------------

fn zh_hant(msg: Msg) -> &'static str {
    match msg {
        Msg::MenuFile => "檔案",
        Msg::MenuEdit => "編輯",
        Msg::MenuView => "檢視",
        Msg::MenuFormat => "格式",
        Msg::MenuExport => "匯出",
        Msg::MenuHelp => "說明",

        Msg::ItemNew => "新增",
        Msg::ItemOpen => "開啟",
        Msg::ItemOpenFolder => "開啟資料夾",
        Msg::ItemSave => "儲存",
        Msg::ItemSaveAs => "另存新檔",
        Msg::ItemNewTab => "新增分頁",
        Msg::ItemOpenInNewTab => "在新分頁開啟",
        Msg::ItemCloseTab => "關閉分頁",
        Msg::ItemNextTab => "下一個分頁",
        Msg::ItemPrevTab => "上一個分頁",
        Msg::ItemExit => "結束",

        Msg::ItemUndo => "復原",
        Msg::ItemRedo => "取消復原",
        Msg::ItemCopy => "複製",
        Msg::ItemCut => "剪下",
        Msg::ItemPaste => "貼上",
        Msg::ItemSelectAll => "全選",
        Msg::ItemPreviewCopyPlain => "複製為純文字",
        Msg::ItemPreviewCopyMarkdown => "複製為 Markdown",
        Msg::ItemPreviewCopyHtml => "複製為 HTML",
        Msg::ItemPreviewSelectAll => "全選",
        Msg::ItemPreviewCopyLinkAddress => "複製連結地址",

        Msg::ItemToggleView => "循環切換檢視模式",
        Msg::ItemEditMode => "編輯模式",
        Msg::ItemVisualEditMode => "視覺化編輯模式",
        Msg::ItemSplitPreviewMode => "分割預覽模式",
        Msg::ItemReadMode => "閱讀模式",
        Msg::ItemToggleSidebar => "切換側邊欄",
        Msg::ItemFiles => "檔案",
        Msg::ItemOutline => "大綱",
        Msg::ItemFocusMode => "專注模式",
        Msg::ItemTypewriterMode => "打字機模式",
        Msg::ItemCodeLineNumbers => "程式碼行號",
        Msg::ItemFind => "尋找",
        Msg::ItemReplace => "取代",
        Msg::ItemFindNext => "尋找下一個",
        Msg::ItemFindPrevious => "尋找上一個",
        Msg::ItemCycleTheme => "切換佈景主題",
        Msg::ItemLanguage => "語言",

        Msg::ItemBold => "粗體",
        Msg::ItemItalic => "斜體",
        Msg::ItemInlineCode => "行內程式碼",
        Msg::ItemLink => "連結",
        Msg::ItemImage => "圖片",
        Msg::ItemH1 => "一級標題",
        Msg::ItemH2 => "二級標題",
        Msg::ItemH3 => "三級標題",
        Msg::ItemH4 => "四級標題",
        Msg::ItemH5 => "五級標題",
        Msg::ItemH6 => "六級標題",
        Msg::ItemBullets => "無序清單",
        Msg::ItemNumbers => "有序清單",
        Msg::ItemTask => "工作清單",
        Msg::ItemQuote => "引用",
        Msg::ItemCodeFence => "程式碼區塊",
        Msg::ItemFormatTable => "格式化表格",
        Msg::ItemAddTableRow => "新增表格列",
        Msg::ItemDeleteTableRow => "刪除表格列",
        Msg::ItemMoveRowUp => "上移表格列",
        Msg::ItemMoveRowDown => "下移表格列",
        Msg::ItemAddTableColumn => "新增表格欄",
        Msg::ItemDeleteTableColumn => "刪除表格欄",

        Msg::ItemExportHtml => "匯出 HTML",
        Msg::ItemExportPlainHtml => "匯出純 HTML",
        Msg::ItemExportPdf => "匯出 PDF",
        Msg::ItemExportLatex => "匯出 LaTeX",
        Msg::ItemExportDocx => "匯出 DOCX",
        Msg::ItemExportPng => "匯出 PNG",
        Msg::ItemExportJpeg => "匯出 JPEG",

        Msg::ItemPreferences => "偏好設定",
        Msg::ItemResetPreferences => "重設偏好設定",
        Msg::ItemKeyboardShortcuts => "鍵盤快速鍵",
        Msg::ItemAboutMarkion => "關於 Markion",

        Msg::LabelEditor => "編輯器",
        Msg::LabelPreview => "預覽",
        Msg::LabelFiles => "檔案",
        Msg::LabelOutline => "大綱",
        Msg::LabelTable => "表格",
        Msg::LabelImageAlt => "替代文字",
        Msg::LabelImageDestination => "目標位址",
        Msg::LabelImageTitle => "標題",

        Msg::SearchFind => "尋找",
        Msg::SearchReplace => "取代",
        Msg::SearchPrev => "上一個",
        Msg::SearchNext => "下一個",
        Msg::SearchAll => "全部",
        Msg::SearchLiteral => "文字",
        Msg::SearchRegexMark => ".*",
        Msg::SearchCaseInsensitiveMark => "aa",
        Msg::SearchCaseSensitiveMark => "Aa",
        Msg::SearchProgress => "{0}/{1}",

        Msg::StatusReady => "就緒",
        Msg::StatusRecoveryAvailable => "有可復原的文件",
        Msg::StatusRecoveredDocument => "已復原未儲存的文件",
        Msg::StatusRecoveryDiscarded => "已放棄復原",
        Msg::StatusWaitingFileTreeConfirm => "等待檔案樹確認…",
        Msg::StatusOpenCanceled => "已取消開啟",
        Msg::StatusOpenFolderCanceled => "已取消開啟資料夾",
        Msg::StatusJumpedToHeading => "已跳至標題",
        Msg::StatusNewDocument => "新文件",
        Msg::StatusOpening => "正在開啟…",
        Msg::StatusOpeningFolder => "正在開啟資料夾…",
        Msg::StatusChoosingSaveLocation => "正在選擇儲存位置…",
        Msg::StatusEditMode => "編輯模式",
        Msg::StatusVisualEditMode => "視覺化編輯模式",
        Msg::StatusSplitPreviewMode => "分割預覽模式",
        Msg::StatusReadMode => "閱讀模式",
        Msg::StatusSidebarShown => "已顯示側邊欄",
        Msg::StatusSidebarHidden => "已隱藏側邊欄",
        Msg::StatusOutlineShown => "已顯示大綱",
        Msg::StatusFileTreeShown => "已顯示檔案樹",
        Msg::StatusFilteringFiles => "正在篩選檔案",
        Msg::StatusFileTreeRefreshed => "已重新整理檔案樹",
        Msg::StatusSelectTreeEntryFirst => "請先選擇一個檔案樹項目",
        Msg::StatusSaveBeforeRename => "重新命名前請先儲存目前文件",
        Msg::StatusWaitingDeleteConfirm => "等待刪除確認…",
        Msg::StatusDeleteCanceled => "已取消刪除",
        Msg::StatusAboutMarkion => "關於 Markion",
        Msg::StatusKeyboardShortcuts => "鍵盤快速鍵",
        Msg::StatusUndo => "已復原",
        Msg::StatusNothingToUndo => "沒有可復原的動作",
        Msg::StatusRedo => "已取消復原",
        Msg::StatusNothingToRedo => "沒有可取消復原的動作",
        Msg::StatusNoFormattingChange => "格式無變化",
        Msg::StatusNoTableAtCursor => "游標處沒有表格",
        Msg::StatusTableAlreadyFormatted => "表格已是格式化的",
        Msg::StatusWaitingConfirm => "等待確認…",
        Msg::StatusCanceled => "已取消",
        Msg::StatusExitingMarkion => "正在結束 Markion",
        Msg::StatusWaitingExitConfirm => "等待結束確認…",
        Msg::StatusExitCanceled => "已取消結束",
        Msg::StatusWaitingQuitConfirm => "等待結束確認…",
        Msg::StatusEditingFindQuery => "正在編輯尋找內容",
        Msg::StatusEditingReplacement => "正在編輯取代內容",
        Msg::StatusEditing => "編輯中",
        Msg::StatusIndentedSelection => "已增加縮排",
        Msg::StatusOutdentedSelection => "已減少縮排",
        Msg::StatusNothingToIndent => "沒有可縮排的內容",
        Msg::StatusNothingToOutdent => "沒有可減少縮排的內容",
        Msg::StatusNoEdit => "無編輯",
        Msg::StatusClipboardEmpty => "剪貼簿為空",
        Msg::StatusCopiedSelection => "已複製所選內容",
        Msg::StatusCopiedPreviewPlain => "已複製為純文字",
        Msg::StatusCopiedPreviewMarkdown => "已複製為 Markdown",
        Msg::StatusCopiedPreviewHtml => "已複製為 HTML",
        Msg::StatusCopiedLinkAddress => "已複製連結地址",
        Msg::StatusPreviewSelectedAll => "已選取全部預覽文字",
        Msg::StatusNothingToCopy => "沒有可複製的內容",
        Msg::StatusCutSelection => "已剪下所選內容",
        Msg::StatusNothingToCut => "沒有可剪下的內容",
        Msg::StatusComposing => "輸入中",
        Msg::StatusSelectedTreeEntry => "已選擇檔案樹項目",
        Msg::StatusNoMatchSelected => "未選取符合項目",
        Msg::StatusReplacedCurrent => "已取代目前符合項目",
        Msg::StatusNoMatchesToReplace => "沒有可取代的符合項目",
        Msg::StatusFindQueryEmpty => "尋找內容為空",
        Msg::StatusNoMatches => "未找到符合項目",

        Msg::StatusFmtBold => "粗體",
        Msg::StatusFmtItalic => "斜體",
        Msg::StatusFmtInlineCode => "行內程式碼",
        Msg::StatusFmtLink => "插入連結",
        Msg::StatusFmtImage => "插入圖片",
        Msg::StatusFmtHeading => "{0} 級標題",
        Msg::StatusFmtBulletedList => "無序清單",
        Msg::StatusFmtNumberedList => "有序清單",
        Msg::StatusFmtTaskList => "工作清單",
        Msg::StatusFmtBlockQuote => "引用",
        Msg::StatusFmtCodeBlock => "程式碼區塊",
        Msg::StatusFmtFormatTable => "格式化表格",
        Msg::StatusFmtAddRow => "新增表格列",
        Msg::StatusFmtDeleteRow => "刪除表格列",
        Msg::StatusFmtMoveRowUp => "上移表格列",
        Msg::StatusFmtMoveRowDown => "下移表格列",
        Msg::StatusFmtAddColumn => "新增表格欄",
        Msg::StatusFmtDeleteColumn => "刪除表格欄",
        Msg::StatusFocusModeOn => "專注模式已開啟",
        Msg::StatusFocusModeOff => "專注模式已關閉",
        Msg::StatusTypewriterModeOn => "打字機模式已開啟",
        Msg::StatusTypewriterModeOff => "打字機模式已關閉",
        Msg::StatusCodeLineNumbersOn => "程式碼行號已開啟",
        Msg::StatusCodeLineNumbersOff => "程式碼行號已關閉",
        Msg::StatusRegexFind => "正規表示式尋找",
        Msg::StatusLiteralFind => "字面尋找",
        Msg::StatusPreferences => "偏好設定",
        Msg::StatusWaitingPreferenceResetConfirm => "等待重設偏好設定確認…",
        Msg::StatusPreferencesReset => "偏好設定已重設",
        Msg::StatusPreviewAdaptiveWidthOn => "預覽自適應寬度已開啟",
        Msg::StatusPreviewAdaptiveWidthOff => "預覽自適應寬度已關閉",
        Msg::StatusSyncScrollOn => "同步捲動已開啟",
        Msg::StatusSyncScrollOff => "同步捲動已關閉",
        Msg::StatusPreferenceResetCanceled => "已取消重設偏好設定",
        Msg::StatusLanguageSet => "已設定語言",

        Msg::StatusRecoveryFailed => "復原失敗：{0}",
        Msg::StatusOpened => "已開啟 {0}",
        Msg::StatusOpenFailed => "開啟失敗：{0}",
        Msg::StatusOpenedFolder => "已開啟資料夾 {0}",
        Msg::StatusOpenFolderFailed => "開啟資料夾失敗：{0}",
        Msg::StatusAutoSaved => "已自動儲存 {0}",
        Msg::StatusRecoverySaved => "已儲存復原副本 {0}",
        Msg::StatusAutoSaveFailed => "自動儲存失敗：{0}",
        Msg::StatusFindFailed => "尋找失敗：{0}",
        Msg::StatusPreferencesSaveFailed => "偏好設定儲存失敗：{0}",
        Msg::StatusSampleThemeSaveFailed => "範例佈景主題儲存失敗：{0}",
        Msg::StatusSaved => "已儲存 {0}",
        Msg::StatusSaveFailed => "儲存失敗：{0}",
        Msg::StatusSaveCanceled => "已取消儲存",
        Msg::StatusExportCanceled => "已取消匯出",
        Msg::StatusChoosingExportLocation => "正在選擇 .{0} 匯出位置…",
        Msg::StatusExported => "已匯出 {0}",
        Msg::StatusExportedEngine => "已匯出 {0}（pandoc 引擎）",
        Msg::StatusExportedBuiltin => {
            "已匯出 {0}（內建簡易匯出——安裝 pandoc 可獲得更高品質的輸出）"
        }
        Msg::StatusExportFailed => "匯出失敗：{0}",
        Msg::StatusCreated => "已建立 {0}",
        Msg::StatusCreateFileFailed => "建立檔案失敗：{0}",
        Msg::StatusCreateFolderFailed => "建立資料夾失敗：{0}",
        Msg::StatusRenamedTo => "已重新命名為 {0}",
        Msg::StatusRenameFailed => "重新命名失敗：{0}",
        Msg::StatusDeleted => "已刪除 {0}",
        Msg::StatusDeleteFailed => "刪除失敗：{0}",
        Msg::StatusShownInFileManager => "已在系統檔案管理員中顯示：{0}",
        Msg::StatusShowInFileManagerFailed => "在系統檔案管理員中顯示失敗：{0}",
        Msg::StatusTheme => "佈景主題：{0}",
        Msg::StatusMatches => "找到 {0} 個符合項目",
        Msg::StatusReplaceFailed => "取代失敗：{0}",
        Msg::StatusReplacedMatches => "已取代 {0} 處符合項目",
        Msg::StatusMatchPosition => "第 {0} 個，共 {1} 個符合項目（{2}:{3}）",
        Msg::StatusFilesVisible => "可見 {0} 個檔案",
        Msg::StatusFileMatches => "{0} 個符合檔案",

        Msg::DialogButtonOk => "確定",
        Msg::DialogButtonCancel => "取消",
        Msg::DialogButtonDiscard => "放棄",
        Msg::DialogButtonDelete => "刪除",
        Msg::DialogButtonReset => "重設",
        Msg::DialogButtonRestore => "復原",
        Msg::DialogButtonExitWithoutSaving => "不儲存結束",

        Msg::DialogAboutTitle => "關於 Markion",
        Msg::DialogAboutDetail => {
            "版本：{0}\n\n一款使用 Rust 與 GPUI 打造的本機優先 Markdown 編輯器。\n\nGitHub：{1}"
        }
        Msg::DialogShortcutsTitle => "鍵盤快速鍵",
        Msg::DialogPreferencesTitle => "偏好設定",
        Msg::DialogPreferencesDetail => PREFERENCES_DETAIL_ZH_HANT,
        Msg::DialogRestoreTitle => "復原未儲存的文件？",
        Msg::DialogRestoreDetail => "Markion 發現了一個未儲存的復原檔案：\n{0}",
        Msg::DialogDiscardTitle => "放棄未儲存的變更？",
        Msg::DialogDiscardNewDetail => "新增文件而不儲存目前的變更。",
        Msg::DialogDiscardOpenDetail => "開啟其他文件而不儲存目前的變更。",
        Msg::DialogDiscardOpenTreeDetail => "開啟 {0} 而不儲存目前的變更。",
        Msg::DialogExitTitle => "不儲存就結束 Markion？",
        Msg::DialogExitDetail => "未儲存的變更將會遺失。",
        Msg::DialogDeleteTitle => "刪除選取的檔案樹項目？",
        Msg::DialogDeleteDetail => "從磁碟刪除 {0}。",
        Msg::DialogDeleteFolderRecursiveTitle => "刪除資料夾及其所有內容？",
        Msg::DialogDeleteFolderRecursiveDetail => "資料夾 {0} 及其中的所有內容將被永久刪除。",
        Msg::DialogResetTitle => "重設偏好設定？",
        Msg::DialogResetDetail => {
            "佈景主題、專注模式、打字機模式、程式碼行號、預覽寬度以及側邊欄設定將還原為預設值。"
        }
        Msg::PromptOpenMarkdown => "開啟 Markdown",
        Msg::PromptOpenFolder => "開啟資料夾",
        Msg::FileTypeMarkdown => "Markdown 文件",
        Msg::FileTypeStyledHtml => "HTML 文件",
        Msg::FileTypePlainHtml => "純 HTML 文件",
        Msg::FileTypePdf => "PDF 文件",
        Msg::FileTypeLatex => "LaTeX 文件",
        Msg::FileTypeDocx => "Word 文件（DOCX）",
        Msg::FileTypePng => "PNG 影像",
        Msg::FileTypeJpeg => "JPEG 影像",

        Msg::SummaryFilesUnit => "個檔案",
        Msg::SummaryMatchesUnit => "個符合項目",

        Msg::FileTreeFilterPlaceholder => "篩選檔案",
        Msg::FileTreeFilterActive => "篩選：{0}",
        Msg::FileTreeWorkspaceFallback => "工作區",
        Msg::FileTreeMoreHidden => "……還有 {0} 項（篩選以縮小範圍）",
        Msg::FileTreeEmptyState => "開啟一個 Markdown 檔案即可在此處查看。",
        Msg::FileTreeContextOpen => "開啟",
        Msg::FileTreeContextOpenInNewTab => "在新分頁中開啟",
        Msg::FileTreeContextCreateFile => "新增檔案",
        Msg::FileTreeContextCreateFolder => "新增資料夾",
        Msg::FileTreeContextRename => "重新命名",
        Msg::FileTreeContextDelete => "刪除",
        Msg::FileTreeContextShowInFileManager => "在系統檔案管理員中顯示",
        Msg::FileTreeContextRefresh => "重新整理",
        Msg::FileTreeContextFilterFiles => "篩選檔案",
        Msg::FileTreeNamePromptLabel => "名稱",
        Msg::StatusNamingEntry => "輸入名稱後按 Enter 確認（Esc 取消）",
        Msg::StatusNameRequired => "名稱不能為空",

        Msg::PrefOn => "開",
        Msg::PrefOff => "關",
        Msg::PrefSidebarHidden => "隱藏",
        Msg::CustomThemeLabel => "{0}（自訂）",

        Msg::PrefPanelTitle => "偏好設定",
        Msg::PrefPanelThemeSection => "佈景主題",
        Msg::PrefPanelLanguageSection => "語言",
        Msg::PrefPanelOtherSection => "其他",
        Msg::PrefPanelFocusMode => "專注模式",
        Msg::PrefPanelTypewriterMode => "打字機模式",
        Msg::PrefPanelCodeLineNumbers => "程式碼行號",
        Msg::PrefPanelPreviewAdaptiveWidth => "預覽自適應寬度",
        Msg::PrefPanelSyncScroll => "同步捲動",
        Msg::PrefPanelSidebar => "側邊欄",
        Msg::PrefPanelHeadingMenu => "標題選單",
        Msg::PrefPanelHeadingMenuThree => "H1–H5",
        Msg::PrefPanelHeadingMenuSix => "H1–H6",
        Msg::PrefPanelClose => "關閉",
        Msg::DiagramLoading => "正在算繪圖表…",
        Msg::DiagramUnsupported => "不支援此圖表格式。",
        Msg::DiagramInputTooLarge => "圖表原始碼超過大小限制。",
        Msg::DiagramInvalidSource => "圖表原始碼無效。",
        Msg::DiagramUnsafeOutput => "基於安全考量，圖表輸出已被封鎖。",
        Msg::DiagramRenderFailed => "圖表算繪失敗。",
        Msg::MathRendering => "正在算繪公式…",
        Msg::MathInvalid => "公式無效。",
        Msg::MathTooLarge => "公式超過算繪大小限制。",
        Msg::MathUnsupported => "公式包含不支援的記法或字形。",
        Msg::MathRenderFailed => "公式算繪失敗。",
        Msg::TitleModified => "已修改",
        Msg::TitleSaved => "已儲存",
    }
}

const PREFERENCES_DETAIL_ZH_HANT: &str = "佈景主題：{0}\n專注模式：{1}\n打字機模式：{2}\n程式碼行號：{3}\n預覽自適應寬度：{4}\n側邊欄：{5}\n\n偏好設定：{6}\n佈景主題目錄：{7}\n已安裝自訂佈景主題：{8}";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn substitute(template: &str, args: &[&str]) -> String {
    if !template.contains('{') {
        return template.to_string();
    }
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' {
            out.push(ch);
            continue;
        }
        // Collect digits until '}'.
        let mut index_str = String::new();
        let mut closed = false;
        while let Some(&next) = chars.peek() {
            if next == '}' {
                chars.next();
                closed = true;
                break;
            }
            if next.is_ascii_digit() {
                index_str.push(next);
                chars.next();
            } else {
                break;
            }
        }
        if closed {
            if let Ok(index) = index_str.parse::<usize>()
                && let Some(value) = args.get(index)
            {
                out.push_str(value);
                continue;
            }
            // Unknown index / missing arg: keep the original token verbatim.
            out.push('{');
            out.push_str(&index_str);
            out.push('}');
        } else {
            // Not a valid placeholder; emit the brace literally.
            out.push('{');
            out.push_str(&index_str);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_round_trips_through_its_code() {
        for &lang in Language::all() {
            assert_eq!(Language::from_code(lang.code()), lang);
        }
    }

    #[test]
    fn language_from_code_accepts_common_aliases() {
        // Simplified-Chinese aliases.
        assert_eq!(Language::from_code("zh"), Language::ZhHans);
        assert_eq!(Language::from_code("zh-CN"), Language::ZhHans);
        assert_eq!(Language::from_code("ZH"), Language::ZhHans);
        assert_eq!(Language::from_code("zh-hans"), Language::ZhHans);
        assert_eq!(Language::from_code("chinese"), Language::ZhHans);
        // Traditional-Chinese aliases.
        assert_eq!(Language::from_code("zh-hant"), Language::ZhHant);
        assert_eq!(Language::from_code("zh-tw"), Language::ZhHant);
        assert_eq!(Language::from_code("cht"), Language::ZhHant);
        assert_eq!(Language::from_code("traditional chinese"), Language::ZhHant);
        assert_eq!(Language::from_code("ja"), Language::Ja);
        assert_eq!(Language::from_code("JP"), Language::Ja);
        assert_eq!(Language::from_code("japanese"), Language::Ja);
        assert_eq!(Language::from_code("fr"), Language::Fr);
        assert_eq!(Language::from_code("francais"), Language::Fr);
        assert_eq!(Language::from_code("french"), Language::Fr);
        assert_eq!(Language::from_code("de"), Language::De);
        assert_eq!(Language::from_code("deutsch"), Language::De);
        assert_eq!(Language::from_code("german"), Language::De);
        assert_eq!(Language::from_code("es"), Language::Es);
        assert_eq!(Language::from_code("espanol"), Language::Es);
        assert_eq!(Language::from_code("spanish"), Language::Es);
        assert_eq!(Language::from_code(""), Language::En);
        assert_eq!(Language::from_code("klingon"), Language::En);
    }

    #[test]
    fn shortcut_platform_defaults_to_the_build_target() {
        assert_eq!(
            ShortcutPlatform::current(),
            if cfg!(target_os = "macos") {
                ShortcutPlatform::MacOS
            } else {
                ShortcutPlatform::WindowsLinux
            }
        );
        assert_eq!(
            ShortcutPlatform::WindowsLinux.label(Language::ZhHans),
            "Windows/Linux"
        );
        assert_eq!(ShortcutPlatform::MacOS.label(Language::Ja), "macOS");
    }

    fn assert_shortcut_action(
        catalog: &ShortcutCatalog,
        category: ShortcutCategory,
        action_label: &str,
        windows_linux: &[&str],
        macos: &[&str],
    ) {
        let section = catalog.section(category).expect("shortcut category");
        let action = section
            .actions
            .iter()
            .find(|action| action.label == action_label)
            .unwrap_or_else(|| panic!("missing shortcut action: {action_label}"));
        assert_eq!(
            action.combinations(ShortcutPlatform::WindowsLinux),
            windows_linux
        );
        assert_eq!(action.combinations(ShortcutPlatform::MacOS), macos);
    }

    #[test]
    fn shortcut_catalog_is_complete_and_platform_explicit_for_every_language() {
        for &language in Language::all() {
            let catalog = shortcut_catalog(language, 3);
            assert_eq!(catalog.sections.len(), ShortcutCategory::ALL.len());
            for (section, category) in catalog.sections.iter().zip(ShortcutCategory::ALL) {
                assert_eq!(section.category, category);
                assert!(!section.label.is_empty());
                assert!(!section.actions.is_empty());
                for action in &section.actions {
                    assert!(!action.label.is_empty());
                    for platform in ShortcutPlatform::ALL {
                        let combinations = action.combinations(platform);
                        assert!(!combinations.is_empty());
                        assert!(combinations.iter().all(|shortcut| {
                            !shortcut.contains("Secondary")
                                && !shortcut.contains("|---")
                                && !shortcut.starts_with('+')
                        }));
                    }
                }
            }
        }
    }

    #[test]
    fn shortcut_catalog_documents_only_ctrl_or_cmd_y_for_redo() {
        for &language in Language::all() {
            for heading_depth in [5, 6] {
                let catalog = shortcut_catalog(language, heading_depth);
                let undo_redo = catalog
                    .section(ShortcutCategory::Editing)
                    .expect("editing shortcut category")
                    .actions
                    .last()
                    .expect("undo/redo shortcut row");
                assert_eq!(
                    undo_redo.combinations(ShortcutPlatform::WindowsLinux),
                    &["Ctrl+Z", "Ctrl+Y"]
                );
                assert_eq!(
                    undo_redo.combinations(ShortcutPlatform::MacOS),
                    &["Cmd+Z", "Cmd+Y"]
                );
            }
        }
    }

    #[test]
    fn english_shortcut_catalog_lists_core_workflows() {
        let catalog = shortcut_catalog(Language::En, 3);
        assert_shortcut_action(
            &catalog,
            ShortcutCategory::Files,
            "Save",
            &["Ctrl+S"],
            &["Cmd+S"],
        );
        assert_shortcut_action(
            &catalog,
            ShortcutCategory::View,
            "Edit Mode",
            &["Ctrl+Alt+1"],
            &["Cmd+Option+1"],
        );
        assert_shortcut_action(
            &catalog,
            ShortcutCategory::View,
            "Sidebar",
            &["Ctrl+Shift+B"],
            &["Cmd+Shift+B"],
        );
        assert_shortcut_action(
            &catalog,
            ShortcutCategory::View,
            "Preferences",
            &["Ctrl+,"],
            &["Cmd+,"],
        );
        assert_shortcut_action(
            &catalog,
            ShortcutCategory::Export,
            "DOCX",
            &["Ctrl+Shift+D"],
            &["Cmd+Shift+D"],
        );
    }

    #[test]
    fn shortcut_catalog_translates_category_and_action_labels() {
        let expectations = [
            (Language::En, "Files", "Save"),
            (Language::ZhHans, "文件", "保存"),
            (Language::ZhHant, "檔案", "儲存"),
            (Language::Ja, "ファイル", "保存"),
            (Language::Fr, "Fichiers", "Enregistrer"),
            (Language::De, "Dateien", "Speichern"),
            (Language::Es, "Archivos", "Guardar"),
        ];
        for (language, category_label, save_label) in expectations {
            let catalog = shortcut_catalog(language, 3);
            let files = catalog.section(ShortcutCategory::Files).unwrap();
            assert_eq!(files.label, category_label);
            assert!(
                files
                    .actions
                    .iter()
                    .any(|action| action.label == save_label)
            );
        }
    }

    #[test]
    fn extended_heading_depth_adds_h6_shortcuts() {
        let standard = shortcut_catalog(Language::En, 5);
        let extended = shortcut_catalog(Language::En, 6);
        let standard_headings = standard.section(ShortcutCategory::Editing).unwrap().actions[5];
        let extended_headings = extended.section(ShortcutCategory::Editing).unwrap().actions[5];
        assert_eq!(
            standard_headings
                .combinations(ShortcutPlatform::WindowsLinux)
                .len(),
            5
        );
        assert_eq!(
            extended_headings
                .combinations(ShortcutPlatform::WindowsLinux)
                .last(),
            Some(&"Ctrl+6")
        );
        assert_eq!(
            extended_headings
                .combinations(ShortcutPlatform::MacOS)
                .last(),
            Some(&"Cmd+6")
        );
    }

    #[test]
    fn substitute_replaces_positional_placeholders() {
        assert_eq!(substitute("{0} matches", &["12"]), "12 matches");
        assert_eq!(substitute("Match {0} of {1}", &["3", "9"]), "Match 3 of 9");
        // No placeholders → no allocation surprises.
        assert_eq!(substitute("plain", &[]), "plain");
        // Out-of-range index keeps the token verbatim.
        assert_eq!(substitute("{0} and {2}", &["a"]), "a and {2}");
    }

    #[test]
    fn tf_uses_template_per_language() {
        assert_eq!(tf(Language::En, Msg::StatusMatches, &["5"]), "5 matches");
        assert_eq!(
            tf(Language::ZhHans, Msg::StatusMatches, &["5"]),
            "找到 5 个匹配"
        );
        assert_eq!(
            tf(Language::ZhHant, Msg::StatusMatches, &["5"]),
            "找到 5 個符合項目"
        );
    }

    #[test]
    fn open_folder_chrome_is_localized() {
        assert_eq!(t(Language::En, Msg::ItemOpenFolder), "Open Folder");
        assert_eq!(t(Language::ZhHans, Msg::ItemOpenFolder), "打开文件夹");
        assert_eq!(t(Language::ZhHant, Msg::ItemOpenFolder), "開啟資料夾");
        assert_eq!(t(Language::Ja, Msg::PromptOpenFolder), "フォルダーを開く");
        assert_eq!(
            t(Language::Fr, Msg::StatusOpeningFolder),
            "Ouverture du dossier..."
        );
        assert_eq!(
            tf(Language::De, Msg::StatusOpenedFolder, &["C:\\Notes"]),
            "Ordner geöffnet: C:\\Notes"
        );
        assert_eq!(
            tf(Language::Es, Msg::StatusOpenFolderFailed, &["denegado"]),
            "Error al abrir la carpeta: denegado"
        );
    }

    #[test]
    fn every_message_returns_non_empty_text_for_every_language() {
        // Exhaustiveness guard: if a new Msg variant is added without a
        // translation arm, this still compiles (the match is total) but a
        // missing arm would be a compile error. Here we sanity-check that no
        // variant accidentally maps to an empty string.
        let all = [
            Msg::MenuFile,
            Msg::MenuEdit,
            Msg::MenuView,
            Msg::MenuFormat,
            Msg::MenuExport,
            Msg::MenuHelp,
            Msg::ItemNew,
            Msg::ItemOpen,
            Msg::ItemOpenFolder,
            Msg::ItemSave,
            Msg::ItemSaveAs,
            Msg::ItemNewTab,
            Msg::ItemOpenInNewTab,
            Msg::ItemCloseTab,
            Msg::ItemNextTab,
            Msg::ItemPrevTab,
            Msg::ItemExit,
            Msg::ItemUndo,
            Msg::ItemRedo,
            Msg::ItemCopy,
            Msg::ItemCut,
            Msg::ItemPaste,
            Msg::ItemSelectAll,
            Msg::ItemPreviewCopyPlain,
            Msg::ItemPreviewCopyMarkdown,
            Msg::ItemPreviewCopyHtml,
            Msg::ItemPreviewSelectAll,
            Msg::ItemPreviewCopyLinkAddress,
            Msg::ItemToggleView,
            Msg::ItemEditMode,
            Msg::ItemVisualEditMode,
            Msg::ItemSplitPreviewMode,
            Msg::ItemReadMode,
            Msg::ItemToggleSidebar,
            Msg::ItemFiles,
            Msg::ItemOutline,
            Msg::ItemFocusMode,
            Msg::ItemTypewriterMode,
            Msg::ItemCodeLineNumbers,
            Msg::ItemFind,
            Msg::ItemReplace,
            Msg::ItemFindNext,
            Msg::ItemFindPrevious,
            Msg::ItemCycleTheme,
            Msg::ItemLanguage,
            Msg::ItemBold,
            Msg::ItemItalic,
            Msg::ItemInlineCode,
            Msg::ItemLink,
            Msg::ItemImage,
            Msg::ItemH1,
            Msg::ItemH2,
            Msg::ItemH3,
            Msg::ItemH4,
            Msg::ItemH5,
            Msg::ItemH6,
            Msg::ItemBullets,
            Msg::ItemNumbers,
            Msg::ItemTask,
            Msg::ItemQuote,
            Msg::ItemCodeFence,
            Msg::ItemFormatTable,
            Msg::ItemAddTableRow,
            Msg::ItemDeleteTableRow,
            Msg::ItemMoveRowUp,
            Msg::ItemMoveRowDown,
            Msg::ItemAddTableColumn,
            Msg::ItemDeleteTableColumn,
            Msg::ItemExportHtml,
            Msg::ItemExportPlainHtml,
            Msg::ItemExportPdf,
            Msg::ItemExportLatex,
            Msg::ItemExportDocx,
            Msg::ItemExportPng,
            Msg::ItemExportJpeg,
            Msg::ItemPreferences,
            Msg::ItemResetPreferences,
            Msg::ItemKeyboardShortcuts,
            Msg::ItemAboutMarkion,
            Msg::LabelEditor,
            Msg::LabelPreview,
            Msg::LabelFiles,
            Msg::LabelOutline,
            Msg::LabelTable,
            Msg::LabelImageAlt,
            Msg::LabelImageDestination,
            Msg::LabelImageTitle,
            Msg::SearchFind,
            Msg::SearchReplace,
            Msg::SearchPrev,
            Msg::SearchNext,
            Msg::SearchAll,
            Msg::SearchLiteral,
            Msg::SearchRegexMark,
            Msg::SearchCaseInsensitiveMark,
            Msg::SearchCaseSensitiveMark,
            Msg::SearchProgress,
            Msg::StatusReady,
            Msg::StatusRecoveryAvailable,
            Msg::StatusRecoveredDocument,
            Msg::StatusRecoveryDiscarded,
            Msg::StatusWaitingFileTreeConfirm,
            Msg::StatusOpenCanceled,
            Msg::StatusOpenFolderCanceled,
            Msg::StatusJumpedToHeading,
            Msg::StatusNewDocument,
            Msg::StatusOpening,
            Msg::StatusOpeningFolder,
            Msg::StatusChoosingSaveLocation,
            Msg::StatusEditMode,
            Msg::StatusVisualEditMode,
            Msg::StatusSplitPreviewMode,
            Msg::StatusReadMode,
            Msg::StatusSidebarShown,
            Msg::StatusSidebarHidden,
            Msg::StatusOutlineShown,
            Msg::StatusFileTreeShown,
            Msg::StatusFilteringFiles,
            Msg::StatusFileTreeRefreshed,
            Msg::StatusSelectTreeEntryFirst,
            Msg::StatusSaveBeforeRename,
            Msg::StatusWaitingDeleteConfirm,
            Msg::StatusDeleteCanceled,
            Msg::StatusAboutMarkion,
            Msg::StatusKeyboardShortcuts,
            Msg::StatusUndo,
            Msg::StatusNothingToUndo,
            Msg::StatusRedo,
            Msg::StatusNothingToRedo,
            Msg::StatusNoFormattingChange,
            Msg::StatusNoTableAtCursor,
            Msg::StatusTableAlreadyFormatted,
            Msg::StatusWaitingConfirm,
            Msg::StatusCanceled,
            Msg::StatusExitingMarkion,
            Msg::StatusWaitingExitConfirm,
            Msg::StatusExitCanceled,
            Msg::StatusWaitingQuitConfirm,
            Msg::StatusEditingFindQuery,
            Msg::StatusEditingReplacement,
            Msg::StatusEditing,
            Msg::StatusIndentedSelection,
            Msg::StatusOutdentedSelection,
            Msg::StatusNothingToIndent,
            Msg::StatusNothingToOutdent,
            Msg::StatusNoEdit,
            Msg::StatusClipboardEmpty,
            Msg::StatusCopiedSelection,
            Msg::StatusCopiedPreviewPlain,
            Msg::StatusCopiedPreviewMarkdown,
            Msg::StatusCopiedPreviewHtml,
            Msg::StatusCopiedLinkAddress,
            Msg::StatusPreviewSelectedAll,
            Msg::StatusNothingToCopy,
            Msg::StatusCutSelection,
            Msg::StatusNothingToCut,
            Msg::StatusComposing,
            Msg::StatusSelectedTreeEntry,
            Msg::StatusNoMatchSelected,
            Msg::StatusReplacedCurrent,
            Msg::StatusNoMatchesToReplace,
            Msg::StatusFindQueryEmpty,
            Msg::StatusNoMatches,
            Msg::StatusFmtBold,
            Msg::StatusFmtItalic,
            Msg::StatusFmtInlineCode,
            Msg::StatusFmtLink,
            Msg::StatusFmtImage,
            Msg::StatusFmtHeading,
            Msg::StatusFmtBulletedList,
            Msg::StatusFmtNumberedList,
            Msg::StatusFmtTaskList,
            Msg::StatusFmtBlockQuote,
            Msg::StatusFmtCodeBlock,
            Msg::StatusFmtFormatTable,
            Msg::StatusFmtAddRow,
            Msg::StatusFmtDeleteRow,
            Msg::StatusFmtMoveRowUp,
            Msg::StatusFmtMoveRowDown,
            Msg::StatusFmtAddColumn,
            Msg::StatusFmtDeleteColumn,
            Msg::StatusFocusModeOn,
            Msg::StatusFocusModeOff,
            Msg::StatusTypewriterModeOn,
            Msg::StatusTypewriterModeOff,
            Msg::StatusCodeLineNumbersOn,
            Msg::StatusCodeLineNumbersOff,
            Msg::StatusRegexFind,
            Msg::StatusLiteralFind,
            Msg::StatusPreferences,
            Msg::StatusWaitingPreferenceResetConfirm,
            Msg::StatusPreferencesReset,
            Msg::StatusPreviewAdaptiveWidthOn,
            Msg::StatusPreviewAdaptiveWidthOff,
            Msg::StatusSyncScrollOn,
            Msg::StatusSyncScrollOff,
            Msg::StatusPreferenceResetCanceled,
            Msg::StatusLanguageSet,
            Msg::StatusRecoveryFailed,
            Msg::StatusOpened,
            Msg::StatusOpenFailed,
            Msg::StatusOpenedFolder,
            Msg::StatusOpenFolderFailed,
            Msg::StatusAutoSaved,
            Msg::StatusRecoverySaved,
            Msg::StatusAutoSaveFailed,
            Msg::StatusFindFailed,
            Msg::StatusPreferencesSaveFailed,
            Msg::StatusSampleThemeSaveFailed,
            Msg::StatusSaved,
            Msg::StatusSaveFailed,
            Msg::StatusSaveCanceled,
            Msg::StatusExportCanceled,
            Msg::StatusChoosingExportLocation,
            Msg::StatusExported,
            Msg::StatusExportFailed,
            Msg::StatusCreated,
            Msg::StatusCreateFileFailed,
            Msg::StatusCreateFolderFailed,
            Msg::StatusRenamedTo,
            Msg::StatusRenameFailed,
            Msg::StatusDeleted,
            Msg::StatusDeleteFailed,
            Msg::StatusShownInFileManager,
            Msg::StatusShowInFileManagerFailed,
            Msg::StatusTheme,
            Msg::StatusMatches,
            Msg::StatusReplaceFailed,
            Msg::StatusReplacedMatches,
            Msg::StatusMatchPosition,
            Msg::StatusFilesVisible,
            Msg::StatusFileMatches,
            Msg::DialogButtonOk,
            Msg::DialogButtonCancel,
            Msg::DialogButtonDiscard,
            Msg::DialogButtonDelete,
            Msg::DialogButtonReset,
            Msg::DialogButtonRestore,
            Msg::DialogButtonExitWithoutSaving,
            Msg::DialogAboutTitle,
            Msg::DialogAboutDetail,
            Msg::DialogShortcutsTitle,
            Msg::DialogPreferencesTitle,
            Msg::DialogPreferencesDetail,
            Msg::DialogRestoreTitle,
            Msg::DialogRestoreDetail,
            Msg::DialogDiscardTitle,
            Msg::DialogDiscardNewDetail,
            Msg::DialogDiscardOpenDetail,
            Msg::DialogDiscardOpenTreeDetail,
            Msg::DialogExitTitle,
            Msg::DialogExitDetail,
            Msg::DialogDeleteTitle,
            Msg::DialogDeleteDetail,
            Msg::DialogDeleteFolderRecursiveTitle,
            Msg::DialogDeleteFolderRecursiveDetail,
            Msg::DialogResetTitle,
            Msg::DialogResetDetail,
            Msg::PromptOpenMarkdown,
            Msg::PromptOpenFolder,
            Msg::FileTypeMarkdown,
            Msg::FileTypeStyledHtml,
            Msg::FileTypePlainHtml,
            Msg::FileTypePdf,
            Msg::FileTypeLatex,
            Msg::FileTypeDocx,
            Msg::FileTypePng,
            Msg::FileTypeJpeg,
            Msg::SummaryFilesUnit,
            Msg::SummaryMatchesUnit,
            Msg::FileTreeFilterPlaceholder,
            Msg::FileTreeFilterActive,
            Msg::FileTreeWorkspaceFallback,
            Msg::FileTreeMoreHidden,
            Msg::FileTreeEmptyState,
            Msg::FileTreeContextOpen,
            Msg::FileTreeContextOpenInNewTab,
            Msg::FileTreeContextCreateFile,
            Msg::FileTreeContextCreateFolder,
            Msg::FileTreeContextRename,
            Msg::FileTreeContextDelete,
            Msg::FileTreeContextShowInFileManager,
            Msg::FileTreeContextRefresh,
            Msg::FileTreeContextFilterFiles,
            Msg::FileTreeNamePromptLabel,
            Msg::StatusNamingEntry,
            Msg::StatusNameRequired,
            Msg::PrefOn,
            Msg::PrefOff,
            Msg::PrefSidebarHidden,
            Msg::CustomThemeLabel,
            Msg::PrefPanelTitle,
            Msg::PrefPanelThemeSection,
            Msg::PrefPanelLanguageSection,
            Msg::PrefPanelOtherSection,
            Msg::PrefPanelFocusMode,
            Msg::PrefPanelTypewriterMode,
            Msg::PrefPanelCodeLineNumbers,
            Msg::PrefPanelPreviewAdaptiveWidth,
            Msg::PrefPanelSyncScroll,
            Msg::PrefPanelSidebar,
            Msg::PrefPanelHeadingMenu,
            Msg::PrefPanelHeadingMenuThree,
            Msg::PrefPanelHeadingMenuSix,
            Msg::PrefPanelClose,
            Msg::TitleModified,
            Msg::TitleSaved,
        ];
        for &msg in all.iter() {
            assert!(!t(Language::En, msg).is_empty(), "empty En for {msg:?}");
            assert!(
                !t(Language::ZhHans, msg).is_empty(),
                "empty ZhHans for {msg:?}"
            );
            assert!(
                !t(Language::ZhHant, msg).is_empty(),
                "empty ZhHant for {msg:?}"
            );
            assert!(!t(Language::Ja, msg).is_empty(), "empty Ja for {msg:?}");
            assert!(!t(Language::Fr, msg).is_empty(), "empty Fr for {msg:?}");
            assert!(!t(Language::De, msg).is_empty(), "empty De for {msg:?}");
            assert!(!t(Language::Es, msg).is_empty(), "empty Es for {msg:?}");
        }
    }
}
