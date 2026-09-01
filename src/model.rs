//! Public domain types shared between the core library and the UI layer.
//!
//! These are plain data structures (with only trivial constructors / trait
//! impls). Behavior on [`MarkdownDocument`](crate::MarkdownDocument) lives in
//! the crate root and the `document` module group.

use std::ops::Range;
use std::path::{Path, PathBuf};

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

/// Default source-editor font size in logical pixels.
pub const DEFAULT_EDITOR_FONT_SIZE: u16 = 14;
/// Smallest supported source-editor font size in logical pixels.
pub const MIN_EDITOR_FONT_SIZE: u16 = 10;
/// Largest supported source-editor font size in logical pixels.
pub const MAX_EDITOR_FONT_SIZE: u16 = 32;

/// Default rendered-document body font size in logical pixels.
pub const DEFAULT_RENDERED_FONT_SIZE: u16 = 14;
/// Smallest supported rendered-document body font size in logical pixels.
pub const MIN_RENDERED_FONT_SIZE: u16 = 10;
/// Largest supported rendered-document body font size in logical pixels.
pub const MAX_RENDERED_FONT_SIZE: u16 = 32;

/// Default gap after a rendered paragraph in logical pixels.
pub const DEFAULT_PARAGRAPH_SPACING: u16 = 12;
/// Smallest supported rendered paragraph gap in logical pixels.
pub const MIN_PARAGRAPH_SPACING: u16 = 0;
/// Largest supported rendered paragraph gap in logical pixels.
pub const MAX_PARAGRAPH_SPACING: u16 = 32;

/// Normalizes a persisted heading-menu depth to the supported values `5` or `6`.
pub fn normalize_heading_menu_max_level(level: u8) -> u8 {
    if level >= EXTENDED_HEADING_MENU_MAX_LEVEL {
        EXTENDED_HEADING_MENU_MAX_LEVEL
    } else {
        DEFAULT_HEADING_MENU_MAX_LEVEL
    }
}

/// Clamps a source-editor font size read from UI or persisted configuration.
pub fn normalize_editor_font_size(value: i64) -> u16 {
    value.clamp(MIN_EDITOR_FONT_SIZE as i64, MAX_EDITOR_FONT_SIZE as i64) as u16
}

/// Clamps a rendered-document body font size read from UI or configuration.
pub fn normalize_rendered_font_size(value: i64) -> u16 {
    value.clamp(MIN_RENDERED_FONT_SIZE as i64, MAX_RENDERED_FONT_SIZE as i64) as u16
}

/// Clamps a rendered paragraph gap read from UI or persisted configuration.
pub fn normalize_paragraph_spacing(value: i64) -> u16 {
    value.clamp(MIN_PARAGRAPH_SPACING as i64, MAX_PARAGRAPH_SPACING as i64) as u16
}

/// Maximum number of paths kept in the recent-files list.
pub const MAX_RECENT_FILES: usize = 10;

/// Persisted editor session: last workspace root, open saved tabs, and recent files.
/// Stored separately from [`AppPreferences`] in `session.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionState {
    pub workspace_root: Option<PathBuf>,
    pub open_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub recent_files: Vec<PathBuf>,
}

impl SessionState {
    /// Move `path` to the front of `recent_files`, deduplicating and capping length.
    pub fn touch_recent(&mut self, path: PathBuf) {
        touch_recent_file(&mut self.recent_files, path, MAX_RECENT_FILES);
    }

    /// Remove `path` from the recent-files list if present.
    pub fn remove_recent(&mut self, path: &Path) {
        self.recent_files.retain(|entry| entry != path);
    }

    /// Clear the recent-files list.
    pub fn clear_recent(&mut self) {
        self.recent_files.clear();
    }
}

/// Insert `path` at the front of `recent`, removing duplicates and truncating to `max`.
pub fn touch_recent_file(recent: &mut Vec<PathBuf>, path: PathBuf, max: usize) {
    recent.retain(|entry| entry != &path);
    recent.insert(0, path);
    if recent.len() > max {
        recent.truncate(max);
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
    /// Source-editor font size in logical pixels.
    pub editor_font_size: u16,
    /// Visual Edit and preview/read body font size in logical pixels.
    pub rendered_font_size: u16,
    /// Bottom gap after rendered paragraph blocks in logical pixels.
    pub paragraph_spacing: u16,
    /// Explicit font family for the Markdown source editor surface. `None`
    /// follows the active theme's `[fonts]` editor entry, then the built-in
    /// default (`.SystemUIFont`).
    pub editor_font_family: Option<String>,
    /// Explicit font family for rendered body text (Visual Edit, Split
    /// Preview's rendered pane, Read mode). `None` follows the active theme's
    /// `[fonts]` rendered entry, then `.SystemUIFont`.
    pub rendered_font_family: Option<String>,
    /// Explicit font family for code surfaces (fenced code blocks, Visual
    /// Edit source islands, reference-definition views). `None` follows the
    /// active theme's `[fonts]` code entry, then "JetBrains Mono".
    pub code_font_family: Option<String>,
    /// Maximum ATX heading level exposed in the Format menu and shortcut
    /// reference. Allowed values are `5` (default) and `6`.
    pub heading_menu_max_level: u8,
    /// When enabled and the active view mode is Split Preview, the source
    /// editor and rendered preview panes stay aligned by Markdown source position.
    /// Disabled by default; no effect in Edit or Read mode.
    pub sync_scroll: bool,
    /// When enabled, the file-tree panel lists hidden entries (dotfile names
    /// on every platform, plus Windows-hidden-attribute entries on Windows).
    /// Disabled by default; the always-excluded build/dependency noise list
    /// (`target`, `node_modules`, …) stays excluded regardless of this flag.
    pub show_hidden_files: bool,
    /// When enabled, opening a supported document or image through a
    /// non-explicit entry (File → Open, file-tree click, drag-and-drop,
    /// Open Recent) replaces the active tab when that is safe (an image,
    /// untitled, or clean document tab); a dirty active tab makes the open
    /// divert to a new tab instead. Enabled by default.
    pub open_in_current_tab: bool,
    pub sidebar_visible: bool,
    pub sidebar_tab: SidebarTab,
    /// Interface language preference code (e.g. "en", "zh"). Stored as a
    /// raw string (not a typed [`crate::i18n::Language`]) to keep `model`
    /// dependency-free; the UI layer interprets it via `Language::from_code`.
    pub language: String,
    /// When enabled, `MarkionApp::new` schedules a silent update check
    /// against the OSS manifest on startup; the dialog appears only if a
    /// newer version is found. Default `false` - no unsolicited network call.
    pub check_for_updates_on_startup: bool,
    /// ISO-8601 timestamp of the most recent update check (manual or
    /// startup). Used to throttle startup checks to at most once per 24h.
    pub last_update_check: Option<String>,
    /// Auto-save behavior. Configurable only via the config file, not the
    /// Preferences panel.
    pub auto_save: AutoSavePreferences,
    /// Export behavior ([export] table). Configurable only via the config
    /// file, not the Preferences panel.
    pub export: ExportPreferences,
    /// Menu-action shortcut overrides ([shortcuts] table): stable action id
    /// -> GPUI keystroke string. Actions without an entry use their default
    /// binding.
    pub shortcut_overrides: std::collections::BTreeMap<String, String>,
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
            editor_font_size: DEFAULT_EDITOR_FONT_SIZE,
            rendered_font_size: DEFAULT_RENDERED_FONT_SIZE,
            paragraph_spacing: DEFAULT_PARAGRAPH_SPACING,
            editor_font_family: None,
            rendered_font_family: None,
            code_font_family: None,
            heading_menu_max_level: DEFAULT_HEADING_MENU_MAX_LEVEL,
            sync_scroll: false,
            show_hidden_files: false,
            open_in_current_tab: true,
            sidebar_visible: true,
            sidebar_tab: SidebarTab::default(),
            language: "en".to_string(),
            check_for_updates_on_startup: false,
            last_update_check: None,
            auto_save: AutoSavePreferences::default(),
            export: ExportPreferences::default(),
            shortcut_overrides: std::collections::BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPreferences {
    /// Which implementation produces PDF/DOCX exports. The default is the
    /// dependency-free built-in writers; `Pandoc` keeps the engine-first flow
    /// with the silent built-in fallback.
    pub backend: ExportBackendPreference,
    /// Pandoc PDF engine used by the engine-first PDF export path
    /// (`--pdf-engine=`), e.g. "xelatex", "pdfroff", "tectonic".
    pub pdf_engine: String,
    /// Explicit pandoc binary path for the engine-first PDF/DOCX export paths.
    /// `None` locates `pandoc` on the system PATH.
    pub pandoc_path: Option<String>,
    /// User-supplied pandoc `--reference-doc` for engine-first DOCX export.
    /// `None` (or a path that does not exist) uses the bundled template.
    pub reference_doc: Option<String>,
    /// Pandoc engine PDF font overrides: `mainfont` and `CJKmainfont`
    /// variables. `None` lets pandoc/xelatex use its own defaults (the
    /// platform CJK default is supplied automatically for CJK documents).
    pub pdf_mainfont: Option<String>,
    pub pdf_cjk_font: Option<String>,
    /// PDF export options ([export.pdf] table), configured on the
    /// Preferences panel Export tab, honored by the built-in PDF writer and
    /// mapped onto pandoc variables on the engine path.
    pub pdf: PdfExportOptions,
    /// DOCX export options ([export.docx] table), configured on the
    /// Preferences panel Export tab and honored by both backends.
    pub docx: DocxExportOptions,
}

impl Default for ExportPreferences {
    fn default() -> Self {
        Self {
            backend: ExportBackendPreference::default(),
            pdf_engine: "xelatex".to_string(),
            pandoc_path: None,
            reference_doc: None,
            pdf_mainfont: None,
            pdf_cjk_font: None,
            pdf: PdfExportOptions::default(),
            docx: DocxExportOptions::default(),
        }
    }
}

/// Which implementation produces PDF/DOCX exports — Markion's built-in
/// writers or the absorbed Typune pandoc engine. This is the persisted
/// preference; the runtime [`ExportBackend`] reports what actually produced
/// a file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExportBackendPreference {
    /// Built-in PDF/DOCX writers produce the file directly; no pandoc
    /// subprocess is spawned (default).
    #[default]
    BuiltIn,
    /// Pandoc engine first, silently falling back to the built-in writers
    /// when the binary is missing or the conversion fails.
    Pandoc,
}

impl ExportBackendPreference {
    /// Config-file token ([export] `backend`).
    pub fn config_value(self) -> &'static str {
        match self {
            Self::BuiltIn => "builtin",
            Self::Pandoc => "pandoc",
        }
    }

    /// Parses a config token; unknown values fall back to BuiltIn.
    pub fn from_config(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "pandoc" | "engine" => Self::Pandoc,
            _ => Self::BuiltIn,
        }
    }
}

/// Page size for PDF export. The built-in writer renders the matching page
/// geometry; the pandoc engine maps it to the geometry variable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PdfPageSize {
    #[default]
    A4,
    Letter,
    Legal,
}

impl PdfPageSize {
    /// (width, height) in millimetres.
    pub fn dimensions_mm(self) -> (f32, f32) {
        match self {
            Self::A4 => (210.0, 297.0),
            Self::Letter => (215.9, 279.4),
            Self::Legal => (215.9, 355.6),
        }
    }

    /// Config-file token ([export.pdf] `page_size`).
    pub fn config_value(self) -> &'static str {
        match self {
            Self::A4 => "a4",
            Self::Letter => "letter",
            Self::Legal => "legal",
        }
    }

    /// Parses a config token; unknown values fall back to A4.
    pub fn from_config(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "letter" => Self::Letter,
            "legal" => Self::Legal,
            _ => Self::A4,
        }
    }
}

/// User-facing PDF export options, persisted as the last-used choices. All
/// four are honored by the built-in PDF writer; the pandoc engine path maps
/// page size to `--variable=geometry:` and `toc` to `--toc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfExportOptions {
    pub page_size: PdfPageSize,
    /// Page margin (all sides) in millimetres.
    pub margin_mm: u32,
    /// Table of contents page built from the document headings.
    pub toc: bool,
    /// Page-number footer on every page.
    pub page_numbers: bool,
}

/// Default PDF page margin in millimetres (2.54 cm).
pub const DEFAULT_PDF_MARGIN_MM: u32 = 25;

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            page_size: PdfPageSize::default(),
            margin_mm: DEFAULT_PDF_MARGIN_MM,
            toc: false,
            page_numbers: true,
        }
    }
}

/// Page size for DOCX export. The built-in writer renders the matching
/// `w:pgSz` dimensions; the pandoc engine maps it to `-V papersize=`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DocxPageSize {
    #[default]
    A4,
    Letter,
    Legal,
}

impl DocxPageSize {
    /// (width, height) in twips; Letter/Legal share the 8.5in width.
    pub fn dimensions_twips(self) -> (u32, u32) {
        match self {
            Self::A4 => (11906, 16838),
            Self::Letter => (12240, 15840),
            Self::Legal => (12240, 20160),
        }
    }

    /// Config-file token ([export.docx] `page_size`).
    pub fn config_value(self) -> &'static str {
        match self {
            Self::A4 => "a4",
            Self::Letter => "letter",
            Self::Legal => "legal",
        }
    }

    /// Parses a config token; unknown values fall back to A4.
    pub fn from_config(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "letter" => Self::Letter,
            "legal" => Self::Legal,
            _ => Self::A4,
        }
    }
}

/// How local images are handled on DOCX export.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DocxImagePolicy {
    /// Embed local images into the package (default).
    #[default]
    Embed,
    /// Export local images as `alt: url` text instead of embedding them.
    TextFallback,
}

impl DocxImagePolicy {
    /// Config-file token ([export.docx] `image_policy`).
    pub fn config_value(self) -> &'static str {
        match self {
            Self::Embed => "embed",
            Self::TextFallback => "text-fallback",
        }
    }

    /// Parses a config token; unknown values fall back to Embed.
    pub fn from_config(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "text-fallback" | "text_fallback" | "text" => Self::TextFallback,
            _ => Self::Embed,
        }
    }
}

/// User-facing DOCX export options, persisted as the last-used choices.
/// `toc` applies only to the pandoc engine path; page size and image policy
/// are honored by both backends.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocxExportOptions {
    pub page_size: DocxPageSize,
    /// Table of contents (pandoc engine only; the built-in writer ignores it).
    pub toc: bool,
    pub image_policy: DocxImagePolicy,
}

/// Which implementation produced an export artifact — the absorbed Typune
/// engine (pandoc subprocess) or Markion's built-in writers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportBackend {
    PandocEngine,
    BuiltIn,
}

/// Why the pandoc engine failed before the built-in writer produced the file.
/// Disclosed on the status bar so a fallback export names the cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineFailureCategory {
    /// The pandoc binary could not be found/launched.
    BinaryMissing,
    /// Pandoc (or the engine parser) ran but the conversion failed.
    ConversionError,
}

/// Outcome of a completed export: which backend produced the file and, when a
/// pandoc-eligible format fell back to the built-in writer, why the engine
/// failed. `engine_failure` is `None` when the engine ran or was not
/// applicable to the format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportOutcome {
    pub backend: ExportBackend,
    pub engine_failure: Option<EngineFailureCategory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoSavePreferences {
    pub enabled: bool,
    /// When true (default), inactivity also writes named documents back to
    /// their file path after the recovery snapshot. When false, only the
    /// recovery snapshot is written and the tab stays dirty.
    pub silent_save: bool,
    /// Inactivity interval before an auto-save fires, in seconds.
    pub delay_secs: u64,
}

/// Minimum inactivity interval accepted for `[auto_save] delay_secs`.
pub const MIN_AUTO_SAVE_DELAY_SECS: u64 = 1;
/// Maximum inactivity interval offered by the Preferences stepper (seconds).
pub const MAX_AUTO_SAVE_DELAY_SECS: u64 = 300;

/// Clamp a raw delay to the supported `[MIN_AUTO_SAVE_DELAY_SECS, MAX_AUTO_SAVE_DELAY_SECS]` range.
pub fn normalize_auto_save_delay_secs(value: i64) -> u64 {
    value.clamp(
        MIN_AUTO_SAVE_DELAY_SECS as i64,
        MAX_AUTO_SAVE_DELAY_SECS as i64,
    ) as u64
}

impl Default for AutoSavePreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            silent_save: true,
            delay_secs: 5,
        }
    }
}

/// Per-slot font-family contributions a theme may carry. `None` means the
/// theme specifies no font for that slot, so the slot falls through to the
/// user preference or the built-in default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThemeFonts {
    pub editor: Option<String>,
    pub rendered: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeDefinition {
    pub name: String,
    pub is_dark: bool,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
}

impl ThemeDefinition {
    /// Constructor for color-only themes (no font contributions) so the
    /// built-in catalog and sample-theme writer stay terse.
    pub fn palette(name: &str, is_dark: bool, colors: ThemeColors) -> Self {
        Self {
            name: name.to_string(),
            is_dark,
            colors,
            fonts: ThemeFonts::default(),
        }
    }
}

/// Font family applied when neither a preference nor a theme names one for
/// the source or rendered slots. gpui resolves this magic name to the
/// platform system UI font (Segoe UI on Windows, SF Pro on macOS).
pub const SYSTEM_UI_FONT_FAMILY: &str = ".SystemUIFont";

/// Built-in default family for the code slot (fenced blocks, source islands,
/// reference-definition views).
pub const DEFAULT_CODE_FONT_FAMILY: &str = "JetBrains Mono";

/// Resolves one font slot: an explicit preference over the active theme's
/// contribution over the built-in default. Empty or whitespace-only values
/// count as unset at every level.
pub fn resolve_font_family(preference: Option<&str>, theme: Option<&str>, default: &str) -> String {
    normalize_font_family(preference)
        .or_else(|| normalize_font_family(theme))
        .unwrap_or_else(|| default.to_string())
}

/// Trims a candidate family name; empty results are treated as absent.
pub fn normalize_font_family(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
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
    #[allow(clippy::too_many_arguments)]
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
        ThemeDefinition::palette(
            "Paper",
            false,
            ThemeColors::new(
                0xf8fafc, 0xffffff, 0xffffff, 0x0f172a, 0x64748b, 0xdbe4ee, 0xe0ecff, 0x1d4ed8,
            ),
        ),
        ThemeDefinition::palette(
            "Ink",
            true,
            ThemeColors::new(
                0x111827, 0x172033, 0x0f172a, 0xe5e7eb, 0x9ca3af, 0x334155, 0x1e3a8a, 0xbfdbfe,
            ),
        ),
        ThemeDefinition::palette(
            "Solar",
            false,
            ThemeColors::new(
                0xfffbeb, 0xffffff, 0xfffdf5, 0x1f2937, 0x78716c, 0xf3d9a4, 0xfef3c7, 0x92400e,
            ),
        ),
        ThemeDefinition::palette(
            "Forest",
            false,
            ThemeColors::new(
                0xf0fdf4, 0xffffff, 0xfafffb, 0x10231a, 0x4b6356, 0xb7ddc2, 0xd1fae5, 0x047857,
            ),
        ),
        ThemeDefinition::palette(
            "Rose",
            false,
            ThemeColors::new(
                0xfff1f2, 0xffffff, 0xfffbfb, 0x2d1720, 0x7f5d65, 0xf5c2cc, 0xffdce5, 0xbe123c,
            ),
        ),
        ThemeDefinition::palette(
            "Graphite",
            false,
            ThemeColors::new(
                0xf4f4f5, 0xffffff, 0xfafafa, 0x18181b, 0x71717a, 0xd4d4d8, 0xe4e4e7, 0x3f3f46,
            ),
        ),
        // --- Popular editor themes ---
        ThemeDefinition::palette(
            "GitHub Light",
            false,
            ThemeColors::new(
                0xffffff, 0xffffff, 0xf6f8fa, 0x24292f, 0x57606a, 0xd0d7de, 0xddf4ff, 0x0969da,
            ),
        ),
        ThemeDefinition::palette(
            "GitHub Dark",
            true,
            ThemeColors::new(
                0x0d1117, 0x161b22, 0x21262d, 0xc9d1d9, 0x8b949e, 0x30363d, 0x1f6feb, 0x58a6ff,
            ),
        ),
        ThemeDefinition::palette(
            "Solarized Light",
            false,
            ThemeColors::new(
                0xfdf6e3, 0xeee8d5, 0xfdf6e3, 0x073642, 0x93a1a1, 0xeee8d5, 0xeee8d5, 0x268bd2,
            ),
        ),
        ThemeDefinition::palette(
            "Solarized Dark",
            true,
            ThemeColors::new(
                0x002b36, 0x073642, 0x073642, 0x93a1a1, 0x586e75, 0x073642, 0x073642, 0x268bd2,
            ),
        ),
        ThemeDefinition::palette(
            "One Light",
            false,
            ThemeColors::new(
                0xfafafa, 0xffffff, 0xf3f3f3, 0x383a42, 0x696c77, 0xe5e5e6, 0xe6f0ff, 0x4078f2,
            ),
        ),
        ThemeDefinition::palette(
            "One Dark",
            true,
            ThemeColors::new(
                0x282c34, 0x21252b, 0x2c313c, 0xabb2bf, 0x5c6370, 0x3b4048, 0x323842, 0x61afef,
            ),
        ),
        ThemeDefinition::palette(
            "Tokyo Night",
            true,
            ThemeColors::new(
                0x1a1b26, 0x16161e, 0x1f2335, 0xc0caf5, 0x565f89, 0x2a2e44, 0x283457, 0x7aa2f7,
            ),
        ),
        ThemeDefinition::palette(
            "Tokyo Night Light",
            false,
            ThemeColors::new(
                0xd5d6db, 0xe1e2e7, 0xcbccd1, 0x343b58, 0x6172b0, 0x9699a3, 0xe1e2e7, 0x34548a,
            ),
        ),
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarTab {
    #[default]
    Files,
    Outline,
}

impl ViewMode {
    pub fn default_mode() -> Self {
        Self::VisualEdit
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
    pub underline: bool,
    pub color: Option<u32>,
}

impl InlineStyle {
    pub fn is_plain(&self) -> bool {
        *self == Self::default()
    }
}

/// A Markdown `![alt](url)` (or equivalent) kept inside a prose construct
/// instead of being extracted as a block-level [`PreviewBlock::Image`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineImage {
    pub alt: String,
    pub url: String,
    pub title: Option<String>,
    pub source_range: Range<usize>,
}

/// A run of preview text sharing one inline style (and optional link target).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InlineSpan {
    pub text: String,
    pub style: InlineStyle,
    pub link: Option<String>,
    pub math: Option<MathSource>,
    pub image: Option<InlineImage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathLayoutStyle {
    Text,
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathDelimiter {
    InlineDollar,
    DisplayDollar,
    Fenced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathSource {
    pub latex: String,
    pub authored: String,
    pub style: MathLayoutStyle,
    pub delimiter: MathDelimiter,
    pub source_range: Range<usize>,
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
    /// Ephemeral identity used to retain row-local UI state across source
    /// versions when this block is proven to be unchanged.
    pub id: VisualBlockId,
    pub kind: VisualBlockKind,
    pub source_range: Range<usize>,
    pub editable_runs: Vec<VisualInlineRun>,
    /// Byte-exact inline constructs that may be temporarily emitted as source
    /// while the rest of the block remains rendered.
    pub reveal_groups: Vec<VisualRevealGroup>,
    pub marker_ranges: Vec<Range<usize>>,
    /// Exact structural prefix for supported line-oriented blocks.
    pub block_prefix: Option<VisualBlockPrefix>,
    /// Rendered-geometry provenance for rows whose height depends on source
    /// content that can change while the block identity stays stable. A
    /// whitespace row carries its covered newline count; every other kind is
    /// `None` because identity already proves its geometry unchanged. The
    /// virtualized list compares this alongside `id` so a height-mutable row
    /// is re-measured instead of reusing a stale cached height.
    pub height_signature: Option<u32>,
    /// Quote decoration and exact source markers inherited from an enclosing
    /// blockquote. The visual row remains a paragraph/list leaf.
    pub quote_context: Option<VisualQuoteContext>,
    pub source_island: Option<VisualSourceIslandKind>,
    /// Exact editable fields for a dedicated complex-block editor. Absent
    /// when the authored syntax cannot be mapped losslessly.
    pub editor: Option<VisualBlockEditor>,
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
    /// Leading `[!NOTE]`-style marker line of a GFM alert quote. Owns exactly
    /// the marker-line bytes with no editable runs: the marker text stays
    /// hidden until focus reveals it through the quote marker mechanism.
    CalloutTitle {
        kind: AlertKind,
    },
    CodeBlock {
        language: Option<String>,
    },
    MathBlock {
        latex: String,
        authored: String,
        delimiter: MathDelimiter,
    },
    Image {
        alt: String,
        url: String,
        title: Option<String>,
    },
    /// Raw HTML block rendered read-only through the shared HTML-parts pipeline
    /// (so `<table>` blocks, text, and images appear). No editable runs and no
    /// source-island — Visual Edit shows the rendered view, not a raw-source box.
    Html {
        html: String,
    },
    Rule,
    Table {
        rows: Vec<Vec<RichText>>,
        alignments: Vec<TableAlignment>,
    },
    /// Footnote definition (`[^label]: …`) covering marker and body.
    FootnoteDefinition {
        label: String,
    },
    /// Standalone link reference definition line(s) (`[label]: url`).
    ReferenceDefinition,
    /// Whitespace-only source not owned by a parsed preview block. Visual Edit
    /// keeps it as a compact row so blank lines and trailing whitespace remain
    /// valid caret positions without showing raw source until focused.
    Whitespace,
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
    /// Resolved navigation destination for an adjacent Visual Edit icon.
    /// Presentation-only: never mutates canonical source.
    pub navigation: Option<VisualNavigationTarget>,
    pub math: Option<MathSource>,
    /// Raw-HTML `<img>` tag presented as an inline image atom. The authored
    /// tag source stays the canonical editable range; this payload is
    /// presentation-only derived data.
    pub html_image: Option<VisualHtmlImage>,
    /// True when the parser's visible text does not map byte-for-byte to source.
    pub conservative_fallback: bool,
}

/// A CSS length authored on an HTML `<img>` `width` or `height` attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlImgLength {
    Px(u32),
    Percent(u16),
}

/// Attributes of one raw-HTML `<img>` tag recognized inside prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualHtmlImage {
    pub alt: String,
    pub url: String,
    pub title: Option<String>,
    pub width: Option<HtmlImgLength>,
    pub height: Option<HtmlImgLength>,
}

/// Destination opened or jumped to from a Visual Edit navigation icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisualNavigationTarget {
    Url(String),
    Footnote { label: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualRevealKind {
    Strong,
    Emphasis,
    Strikethrough,
    InlineCode,
    Link,
    Highlight,
    Superscript,
    Subscript,
    Math,
    /// One complete raw-HTML `<img …>` tag rendered as an inline image atom.
    HtmlImage,
    /// One backslash-escaped ASCII punctuation character (`\*`, `\\`): the
    /// escaped character renders literally while the backslash stays hidden.
    Escape,
    /// One decoded HTML entity reference (`&amp;`, `&#39;`, `&#x2014;`): the
    /// decoded character renders while the complete authored `&…;` token
    /// stays hidden until the caret enters it.
    Entity,
    /// One supported inline-HTML element (style pair or `<br>`): the tags stay
    /// hidden markers while the content renders with the mapped style.
    InlineHtml,
}

/// Opaque, non-persisted identity of a source-backed Visual Edit block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VisualBlockId(pub(crate) u64);

impl VisualBlockId {
    pub(crate) fn fresh() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    /// Stable widget key for this process-local derived block identity.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisualBlockEditor {
    Code {
        opening_fence: Range<usize>,
        payload: VisualEditorField,
        info_range: Option<Range<usize>>,
        closing_fence: Range<usize>,
    },
    Math {
        opening_delimiter: Range<usize>,
        payload: VisualEditorField,
        closing_delimiter: Range<usize>,
    },
    Table {
        cells: Vec<VisualTableCell>,
    },
    Html {
        payload: VisualEditorField,
    },
}

impl VisualBlockEditor {
    pub fn fields(&self) -> Vec<&VisualEditorField> {
        match self {
            Self::Code { payload, .. } | Self::Math { payload, .. } | Self::Html { payload } => {
                vec![payload]
            }
            Self::Table { cells } => cells.iter().map(|cell| &cell.field).collect(),
        }
    }

    pub fn field_containing(&self, range: &Range<usize>) -> Option<&VisualEditorField> {
        self.fields().into_iter().find(|field| {
            range.start >= field.source_range.start && range.end <= field.source_range.end
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualEditorFieldKind {
    CodePayload,
    MathPayload,
    HtmlSource,
    ImageAlt,
    ImageDestination,
    ImageTitle,
    TableCell { row: usize, column: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualEditorField {
    pub kind: VisualEditorFieldKind,
    pub source_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualTableCell {
    pub row: usize,
    pub column: usize,
    pub field: VisualEditorField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualBlockEdit {
    pub document_version: u64,
    pub block_id: VisualBlockId,
    /// The exact current-version field that originated this edit. This is
    /// revalidated immediately before mutation, including for table edits
    /// whose canonical replacement range is the complete table block.
    pub field: VisualEditorField,
    pub range: Range<usize>,
    pub replacement: String,
    /// Exact inserted/composing bytes after applying `replacement`.
    pub inserted_range_after: Range<usize>,
    pub selection_after: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualRevealGroup {
    pub kind: VisualRevealKind,
    /// Complete source syntax, including opening/closing markers and, for a
    /// link, its destination and optional title.
    pub source_range: Range<usize>,
    /// Exact rendered-content ranges contained by this syntax group.
    pub content_ranges: Vec<Range<usize>>,
    pub link_target_range: Option<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualProjectionSegment {
    pub display_range: Range<usize>,
    pub source_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualProjectionSpan {
    pub display_range: Range<usize>,
    pub style: InlineStyle,
    pub link: bool,
    pub source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualCaretAffinity {
    Upstream,
    Downstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualBoundaryCandidates {
    pub display_offset: usize,
    pub upstream_source: usize,
    pub downstream_source: usize,
}

impl VisualBoundaryCandidates {
    pub fn is_ambiguous(self) -> bool {
        self.upstream_source != self.downstream_source
    }

    pub fn resolve(self, affinity: VisualCaretAffinity) -> usize {
        match affinity {
            VisualCaretAffinity::Upstream => self.upstream_source,
            VisualCaretAffinity::Downstream => self.downstream_source,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualProjection {
    pub text: String,
    pub segments: Vec<VisualProjectionSegment>,
    pub spans: Vec<VisualProjectionSpan>,
    pub revealed_source_ranges: Vec<Range<usize>>,
    pub source_anchor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualBlockPrefixKind {
    Heading { level: u8 },
    BlockQuote { depth: usize },
    UnorderedList { level: usize },
    OrderedList { level: usize, index: u64 },
    TaskList { level: usize, checked: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualBlockPrefix {
    pub kind: VisualBlockPrefixKind,
    /// Indentation only. Empty for a top-level block.
    pub indentation_range: Range<usize>,
    /// Indentation plus the complete structural marker and following spacing.
    pub source_range: Range<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualQuoteGroupEdge {
    Only,
    First,
    Middle,
    Last,
}

/// Source-backed quote metadata attached to an ordinary visual leaf row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualQuoteContext {
    pub depth: usize,
    /// Every quote-marker prefix intersecting this row, one per physical line.
    pub marker_ranges: Vec<Range<usize>>,
    /// Semantic source emitted for the underlying paragraph or list item.
    pub leaf_source_range: Range<usize>,
    /// Source range of the containing quote, used to keep adjacent rows in one
    /// visual group without reparsing during rendering.
    pub group_source_range: Range<usize>,
    pub edge: VisualQuoteGroupEdge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualStructuralEdit {
    pub range: Range<usize>,
    pub replacement: String,
    pub selection_after: Range<usize>,
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
                math: None,
                image: None,
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

/// GFM alert kind reported by the parser when a blockquote opens with a
/// `[!NOTE]`-style marker line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
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
        /// Ordered leaf blocks authored inside the quote. Paragraphs and list
        /// items remain distinct so every source byte has exactly one visual
        /// owner and consumers preserve the authored block order.
        children: Vec<PreviewBlock>,
        /// GFM alert kind when the quote opens with a `[!NOTE]`-style marker
        /// line; `None` for plain blockquotes. The marker line is block
        /// structure (no inline events own its bytes), so consumers must
        /// account for it separately from `children`.
        alert: Option<AlertKind>,
        source_range: Range<usize>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
        source_range: Range<usize>,
    },
    MathBlock {
        latex: String,
        /// Byte-identical authored syntax, including delimiters or fence.
        authored: String,
        delimiter: MathDelimiter,
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
        rows: Vec<Vec<RichText>>,
        /// Per-column alignment from the separator row, as parsed upstream.
        alignments: Vec<TableAlignment>,
        source_range: Range<usize>,
    },
    FootnoteDefinition {
        label: String,
        text: RichText,
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
            | Self::Table { source_range, .. }
            | Self::FootnoteDefinition { source_range, .. } => source_range,
        }
    }

    /// Plain-text content of the block, including text nested inside child
    /// blocks (currently list items inside a blockquote).
    pub fn plain_text(&self) -> String {
        match self {
            Self::Heading { text, .. }
            | Self::Paragraph { text, .. }
            | Self::ListItem { text, .. }
            | Self::FootnoteDefinition { text, .. } => text.text.clone(),
            Self::BlockQuote { children, .. } => children
                .iter()
                .map(PreviewBlock::plain_text)
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
            Self::CodeBlock { code, .. } => code.clone(),
            Self::MathBlock { latex, .. } => latex.clone(),
            Self::Html { html, .. } => html.clone(),
            Self::Image { alt, .. } => alt.clone(),
            Self::Rule { .. } => String::new(),
            Self::Table { rows, .. } => rows
                .iter()
                .flat_map(|row| row.iter())
                .map(|cell| cell.text.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
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
    pub disk_identity: Option<crate::DiskIdentity>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutosaveOutcome {
    NoChanges,
    SavedFile(PathBuf),
    SavedRecovery(PathBuf),
}
