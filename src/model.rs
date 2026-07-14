//! Public domain types shared between the core library and the UI layer.
//!
//! These are plain data structures (with only trivial constructors / trait
//! impls). Behavior on [`MarkdownDocument`](crate::MarkdownDocument) lives in
//! the crate root and the `document` module group.

use std::ops::Range;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Markdown,
    Html,
    PlainHtml,
    Pdf,
    Latex,
    Docx,
    Png,
    Jpeg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub title: String,
    pub anchor: String,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Footnote {
    pub label: String,
    pub text: String,
    pub references: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOptions {
    pub query: String,
    pub case_sensitive: bool,
    pub regex: bool,
}

impl SearchOptions {
    pub fn literal(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            case_sensitive: false,
            regex: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatchRange {
    pub range: Range<usize>,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceResult {
    pub replacements: usize,
    pub selected_range: Option<Range<usize>>,
}

/// Default Format-menu heading depth (H1–H5, matching Markion's visible Format menu).
pub const DEFAULT_HEADING_MENU_MAX_LEVEL: u8 = 5;

/// Extended Format-menu heading depth (H1–H6).
pub const EXTENDED_HEADING_MENU_MAX_LEVEL: u8 = 6;

/// Normalizes a persisted heading-menu depth to the supported values `5` or `6`.
pub fn normalize_heading_menu_max_level(level: u8) -> u8 {
    if level >= EXTENDED_HEADING_MENU_MAX_LEVEL {
        EXTENDED_HEADING_MENU_MAX_LEVEL
    } else {
        DEFAULT_HEADING_MENU_MAX_LEVEL
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPreferences {
    pub theme: String,
    pub custom_theme: Option<String>,
    pub focus_mode: bool,
    pub typewriter_mode: bool,
    pub code_line_numbers: bool,
    pub preview_adaptive_width: bool,
    /// Maximum ATX heading level exposed in the Format menu and shortcut
    /// reference. Allowed values are `5` (default) and `6`.
    pub heading_menu_max_level: u8,
    /// When enabled and the active view mode is Split Preview, the source
    /// editor and rendered preview panes scroll together proportionally.
    /// Disabled by default; no effect in Edit or Read mode.
    pub sync_scroll: bool,
    pub sidebar_visible: bool,
    pub sidebar_tab: SidebarTab,
    /// Interface language preference code (e.g. "en", "zh"). Stored as a
    /// raw string (not a typed [`crate::i18n::Language`]) to keep `model`
    /// dependency-free; the UI layer interprets it via `Language::from_code`.
    pub language: String,
    /// Auto-save behavior. Configurable only via the config file, not the
    /// Preferences panel.
    pub auto_save: AutoSavePreferences,
    /// Export behavior ([export] table). Configurable only via the config
    /// file, not the Preferences panel.
    pub export: ExportPreferences,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: "Paper".to_string(),
            custom_theme: None,
            focus_mode: false,
            typewriter_mode: false,
            code_line_numbers: true,
            preview_adaptive_width: false,
            heading_menu_max_level: DEFAULT_HEADING_MENU_MAX_LEVEL,
            sync_scroll: false,
            sidebar_visible: true,
            sidebar_tab: SidebarTab::default(),
            language: "en".to_string(),
            auto_save: AutoSavePreferences::default(),
            export: ExportPreferences::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPreferences {
    /// Pandoc PDF engine used by the engine-first PDF export path
    /// (`--pdf-engine=`), e.g. "xelatex", "pdfroff", "tectonic".
    pub pdf_engine: String,
}

impl Default for ExportPreferences {
    fn default() -> Self {
        Self {
            pdf_engine: "xelatex".to_string(),
        }
    }
}

/// Which implementation produced an export artifact — the absorbed Typune
/// engine (pandoc subprocess) or Markion's built-in writers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportBackend {
    PandocEngine,
    BuiltIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoSavePreferences {
    pub enabled: bool,
    /// Inactivity interval before an auto-save fires, in seconds.
    pub delay_secs: u64,
}

impl Default for AutoSavePreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            delay_secs: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeDefinition {
    pub name: String,
    pub is_dark: bool,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    pub app_bg: u32,
    pub panel_bg: u32,
    pub surface_bg: u32,
    pub text: u32,
    pub muted: u32,
    pub border: u32,
    pub active_bg: u32,
    pub active_text: u32,
}

impl ThemeColors {
    /// Convenience constructor so the built-in theme table reads as labelled
    /// hex values instead of positional struct fields.
    const fn new(
        app_bg: u32,
        panel_bg: u32,
        surface_bg: u32,
        text: u32,
        muted: u32,
        border: u32,
        active_bg: u32,
        active_text: u32,
    ) -> Self {
        Self {
            app_bg,
            panel_bg,
            surface_bg,
            text,
            muted,
            border,
            active_bg,
            active_text,
        }
    }
}

/// All built-in themes, each expressed as a [`ThemeDefinition`] so the
/// Preferences panel can present them alongside user-loaded `.theme` files in
/// a single list. Names are stable identity keys (saved to the preferences
/// file), so renames here would orphan existing saved selections.
///
/// The first six (Paper/Ink/Solar/Forest/Rose/Graphite) predate this table —
/// they are the original `AppTheme` variants and must stay first and in this
/// order so the legacy `cycle_theme` / `app_theme_cycles_through_six_builtin_
/// themes` test keeps passing.
pub fn builtin_theme_definitions() -> Vec<ThemeDefinition> {
    vec![
        // --- Original six built-ins (do not reorder/renumber) ---
        ThemeDefinition {
            name: "Paper".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xf8fafc, 0xffffff, 0xffffff, 0x0f172a, 0x64748b, 0xdbe4ee, 0xe0ecff, 0x1d4ed8,
            ),
        },
        ThemeDefinition {
            name: "Ink".to_string(),
            is_dark: true,
            colors: ThemeColors::new(
                0x111827, 0x172033, 0x0f172a, 0xe5e7eb, 0x9ca3af, 0x334155, 0x1e3a8a, 0xbfdbfe,
            ),
        },
        ThemeDefinition {
            name: "Solar".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xfffbeb, 0xffffff, 0xfffdf5, 0x1f2937, 0x78716c, 0xf3d9a4, 0xfef3c7, 0x92400e,
            ),
        },
        ThemeDefinition {
            name: "Forest".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xf0fdf4, 0xffffff, 0xfafffb, 0x10231a, 0x4b6356, 0xb7ddc2, 0xd1fae5, 0x047857,
            ),
        },
        ThemeDefinition {
            name: "Rose".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xfff1f2, 0xffffff, 0xfffbfb, 0x2d1720, 0x7f5d65, 0xf5c2cc, 0xffdce5, 0xbe123c,
            ),
        },
        ThemeDefinition {
            name: "Graphite".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xf4f4f5, 0xffffff, 0xfafafa, 0x18181b, 0x71717a, 0xd4d4d8, 0xe4e4e7, 0x3f3f46,
            ),
        },
        // --- Popular editor themes ---
        ThemeDefinition {
            name: "GitHub Light".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xffffff, 0xffffff, 0xf6f8fa, 0x24292f, 0x57606a, 0xd0d7de, 0xddf4ff, 0x0969da,
            ),
        },
        ThemeDefinition {
            name: "GitHub Dark".to_string(),
            is_dark: true,
            colors: ThemeColors::new(
                0x0d1117, 0x161b22, 0x21262d, 0xc9d1d9, 0x8b949e, 0x30363d, 0x1f6feb, 0x58a6ff,
            ),
        },
        ThemeDefinition {
            name: "Solarized Light".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xfdf6e3, 0xeee8d5, 0xfdf6e3, 0x073642, 0x93a1a1, 0xeee8d5, 0xeee8d5, 0x268bd2,
            ),
        },
        ThemeDefinition {
            name: "Solarized Dark".to_string(),
            is_dark: true,
            colors: ThemeColors::new(
                0x002b36, 0x073642, 0x073642, 0x93a1a1, 0x586e75, 0x073642, 0x073642, 0x268bd2,
            ),
        },
        ThemeDefinition {
            name: "One Light".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xfafafa, 0xffffff, 0xf3f3f3, 0x383a42, 0x696c77, 0xe5e5e6, 0xe6f0ff, 0x4078f2,
            ),
        },
        ThemeDefinition {
            name: "One Dark".to_string(),
            is_dark: true,
            colors: ThemeColors::new(
                0x282c34, 0x21252b, 0x2c313c, 0xabb2bf, 0x5c6370, 0x3b4048, 0x323842, 0x61afef,
            ),
        },
        ThemeDefinition {
            name: "Tokyo Night".to_string(),
            is_dark: true,
            colors: ThemeColors::new(
                0x1a1b26, 0x16161e, 0x1f2335, 0xc0caf5, 0x565f89, 0x2a2e44, 0x283457, 0x7aa2f7,
            ),
        },
        ThemeDefinition {
            name: "Tokyo Night Light".to_string(),
            is_dark: false,
            colors: ThemeColors::new(
                0xd5d6db, 0xe1e2e7, 0xcbccd1, 0x343b58, 0x6172b0, 0x9699a3, 0xe1e2e7, 0x34548a,
            ),
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathExpression {
    pub latex: String,
    pub display: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedMath {
    pub latex: String,
    pub display: bool,
    pub text: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchError {
    pub(crate) message: String,
}

impl SearchError {
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SearchError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Edit,
    VisualEdit,
    Split,
    Read,
}

/// Which panel the unified sidebar is currently showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    Files,
    Outline,
}

impl Default for SidebarTab {
    fn default() -> Self {
        Self::Files
    }
}

impl ViewMode {
    pub fn default_mode() -> Self {
        Self::Split
    }

    pub fn next(self) -> Self {
        match self {
            Self::Edit => Self::VisualEdit,
            Self::VisualEdit => Self::Split,
            Self::Split => Self::Read,
            Self::Read => Self::Edit,
        }
    }
}

impl Default for ViewMode {
    fn default() -> Self {
        Self::default_mode()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentStats {
    pub bytes: usize,
    pub chars: usize,
    pub words: usize,
    pub lines: usize,
    pub headings: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    Plain,
    Keyword,
    String,
    Number,
    Comment,
    Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedSpan {
    pub text: String,
    pub kind: HighlightKind,
}

/// Inline formatting flags carried by a [`InlineSpan`]. Multiple flags can be
/// active at once (e.g. bold italic inside a highlight).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub highlight: bool,
    pub superscript: bool,
    pub subscript: bool,
}

impl InlineStyle {
    pub fn is_plain(&self) -> bool {
        *self == Self::default()
    }
}

/// A run of preview text sharing one inline style (and optional link target).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InlineSpan {
    pub text: String,
    pub style: InlineStyle,
    pub link: Option<String>,
}

/// Block-level preview text with resolved inline styling. `text` is always the
/// concatenation of `spans`, so consumers that only need plain text (LaTeX,
/// DOCX, tests) can keep using `text`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RichText {
    pub text: String,
    pub spans: Vec<InlineSpan>,
}

/// Source-backed block rendered by Visual Edit. Unlike [`PreviewBlock`], this
/// representation keeps the byte ranges needed to send edits back to the
/// canonical Markdown text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualBlock {
    pub kind: VisualBlockKind,
    pub source_range: Range<usize>,
    pub editable_runs: Vec<VisualInlineRun>,
    pub marker_ranges: Vec<Range<usize>>,
    pub source_island: Option<VisualSourceIslandKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisualBlockKind {
    Heading {
        level: u8,
    },
    Paragraph,
    ListItem {
        level: usize,
        ordered: bool,
        index: Option<u64>,
        checked: Option<bool>,
    },
    BlockQuote,
    CodeBlock {
        language: Option<String>,
    },
    MathBlock,
    Image {
        alt: String,
        url: String,
        title: Option<String>,
    },
    Rule,
    Table {
        rows: Vec<Vec<String>>,
        alignments: Vec<TableAlignment>,
    },
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualInlineRun {
    pub visible_text: String,
    /// Source extent emitted by the Markdown parser for this run.
    pub source_range: Range<usize>,
    /// Exact editable content within `source_range` when it can be identified.
    pub content_range: Range<usize>,
    pub style: InlineStyle,
    pub link_target_range: Option<Range<usize>>,
    /// True when the parser's visible text does not map byte-for-byte to source.
    pub conservative_fallback: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualSourceIslandKind {
    FrontMatter,
    Code,
    Math,
    Html,
    Image,
    Table,
    Unsupported,
}

impl RichText {
    pub fn plain(text: impl Into<String>) -> Self {
        let text = text.into();
        if text.is_empty() {
            return Self::default();
        }
        Self {
            spans: vec![InlineSpan {
                text: text.clone(),
                style: InlineStyle::default(),
                link: None,
            }],
            text,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl From<&str> for RichText {
    fn from(text: &str) -> Self {
        Self::plain(text)
    }
}

impl From<String> for RichText {
    fn from(text: String) -> Self {
        Self::plain(text)
    }
}

impl std::fmt::Display for RichText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewBlock {
    Heading {
        level: u8,
        text: RichText,
        source_range: Range<usize>,
    },
    Paragraph {
        text: RichText,
        source_range: Range<usize>,
    },
    ListItem {
        level: usize,
        ordered: bool,
        /// 1-based number shown for ordered list items, honoring the list's
        /// start attribute (e.g. `3.` continues as 4, 5, ...).
        index: Option<u64>,
        checked: Option<bool>,
        text: RichText,
        source_range: Range<usize>,
    },
    BlockQuote {
        text: RichText,
        source_range: Range<usize>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
        source_range: Range<usize>,
    },
    MathBlock {
        latex: String,
        error: Option<String>,
        source_range: Range<usize>,
    },
    Html {
        html: String,
        source_range: Range<usize>,
    },
    Image {
        alt: String,
        url: String,
        title: Option<String>,
        source_range: Range<usize>,
    },
    Rule {
        source_range: Range<usize>,
    },
    Table {
        rows: Vec<Vec<String>>,
        /// Per-column alignment from the separator row, as parsed upstream.
        alignments: Vec<TableAlignment>,
        source_range: Range<usize>,
    },
}

impl PreviewBlock {
    /// Byte range of this block in the original document text.
    pub fn source_range(&self) -> &Range<usize> {
        match self {
            Self::Heading { source_range, .. }
            | Self::Paragraph { source_range, .. }
            | Self::ListItem { source_range, .. }
            | Self::BlockQuote { source_range, .. }
            | Self::CodeBlock { source_range, .. }
            | Self::MathBlock { source_range, .. }
            | Self::Html { source_range, .. }
            | Self::Image { source_range, .. }
            | Self::Rule { source_range }
            | Self::Table { source_range, .. } => source_range,
        }
    }
}

/// Column alignment declared by a Markdown table separator row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAlignment {
    Default,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownFormat {
    Bold,
    Italic,
    InlineCode,
    Link,
    Image,
    Heading(u8),
    UnorderedList,
    OrderedList,
    TaskList,
    BlockQuote,
    CodeFence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableEdit {
    Format,
    AddRow,
    DeleteRow,
    MoveRowUp,
    MoveRowDown,
    AddColumn,
    DeleteColumn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableEditResult {
    pub table_range: Range<usize>,
    pub selected_range: Range<usize>,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YamlFrontMatter {
    pub raw: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub values: serde_yaml::Mapping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontMatterError {
    pub(crate) message: String,
}

impl FrontMatterError {
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for FrontMatterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FrontMatterError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryDocument {
    pub original_path: Option<PathBuf>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutosaveOutcome {
    NoChanges,
    SavedFile(PathBuf),
    SavedRecovery(PathBuf),
}
