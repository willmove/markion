//! App preference parsing, rendering, and persistence.
//!
//! Preferences persist as TOML (`config.toml`), a design adopted from
//! Typune's filesystem crate: every field is optional and defaulted, and
//! auto-save behavior lives in an `[auto_save]` table. The retired
//! hand-written `key=value` format (`preferences.conf`) is still readable so
//! `load_app_preferences` can migrate it to TOML once, after which the legacy
//! file is ignored.

use std::{collections::BTreeMap, fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::model::{
    AppPreferences, AutoSavePreferences, CodeTheme, DEFAULT_EDITOR_FONT_SIZE,
    DEFAULT_PARAGRAPH_SPACING, DEFAULT_RENDERED_FONT_SIZE, DocxExportOptions, DocxImagePolicy,
    DocxPageSize, ExportBackendPreference, ExportPreferences, PdfExportOptions, PdfPageSize,
    SidebarTab, normalize_code_font_size, normalize_editor_font_size,
    normalize_heading_menu_max_level, normalize_paragraph_spacing, normalize_rendered_font_size,
};

/// File name of the retired `key=value` preferences format, looked for next
/// to the TOML file during migration.
const LEGACY_PREFERENCES_FILE_NAME: &str = "preferences.conf";

/// Serde-facing shape of `config.toml`. Kept separate so `model` stays
/// dependency-free. `#[serde(default)]` on the struct makes every field
/// optional; defaults mirror [`AppPreferences::default`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct PreferencesFile {
    theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_theme: Option<String>,
    language: String,
    #[serde(default)]
    check_for_updates_on_startup: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_update_check: Option<String>,
    focus_mode: bool,
    typewriter_mode: bool,
    code_line_numbers: bool,
    /// "light" | "dark"; unknown values fall back to dark.
    code_theme: String,
    #[serde(deserialize_with = "deserialize_bool_or_true")]
    code_long_line_wrap: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_code_font_size"
    )]
    code_font_size: Option<u16>,
    #[serde(deserialize_with = "deserialize_bool_or_false")]
    preview_adaptive_width: bool,
    #[serde(deserialize_with = "deserialize_editor_font_size")]
    editor_font_size: u16,
    #[serde(deserialize_with = "deserialize_rendered_font_size")]
    rendered_font_size: u16,
    #[serde(deserialize_with = "deserialize_paragraph_spacing")]
    paragraph_spacing: u16,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_font_family"
    )]
    editor_font_family: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_font_family"
    )]
    rendered_font_family: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_font_family"
    )]
    code_font_family: Option<String>,
    #[serde(default = "default_heading_menu_max_level")]
    heading_menu_max_level: u8,
    #[serde(deserialize_with = "deserialize_bool_or_false")]
    sync_scroll: bool,
    #[serde(deserialize_with = "deserialize_bool_or_false")]
    show_hidden_files: bool,
    #[serde(deserialize_with = "deserialize_bool_or_true")]
    open_in_current_tab: bool,
    sidebar_visible: bool,
    /// "files" or "outline"; unknown values fall back to Files like the
    /// legacy format did.
    sidebar_tab: String,
    auto_save: AutoSaveFile,
    export: ExportFile,
    /// [shortcuts] table: action id -> GPUI keystroke string.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    shortcuts: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct AutoSaveFile {
    enabled: bool,
    /// Omitted / invalid values fall back to true (preserve historical write-back).
    #[serde(deserialize_with = "deserialize_bool_or_true")]
    silent_save: bool,
    delay_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct ExportFile {
    /// "builtin" | "pandoc"; unknown values fall back to builtin.
    backend: String,
    pdf_engine: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string"
    )]
    pandoc_path: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string"
    )]
    reference_doc: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string"
    )]
    pdf_mainfont: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string"
    )]
    pdf_cjk_font: Option<String>,
    pdf: PdfExportFile,
    docx: DocxExportFile,
}

/// [export.pdf] table: last-used PDF export options.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct PdfExportFile {
    /// "a4" | "letter" | "legal"; unknown values fall back to A4.
    page_size: String,
    /// Page margin in millimetres; non-numeric values fall back to the default.
    #[serde(deserialize_with = "deserialize_pdf_margin_mm")]
    margin_mm: u32,
    #[serde(deserialize_with = "deserialize_bool_or_false")]
    toc: bool,
    #[serde(deserialize_with = "deserialize_bool_or_true")]
    page_numbers: bool,
}

impl Default for PdfExportFile {
    fn default() -> Self {
        let defaults = PdfExportOptions::default();
        Self {
            page_size: defaults.page_size.config_value().to_string(),
            margin_mm: defaults.margin_mm,
            toc: defaults.toc,
            page_numbers: defaults.page_numbers,
        }
    }
}

impl From<&PdfExportOptions> for PdfExportFile {
    fn from(options: &PdfExportOptions) -> Self {
        Self {
            page_size: options.page_size.config_value().to_string(),
            margin_mm: options.margin_mm,
            toc: options.toc,
            page_numbers: options.page_numbers,
        }
    }
}

impl From<PdfExportFile> for PdfExportOptions {
    fn from(file: PdfExportFile) -> Self {
        Self {
            page_size: PdfPageSize::from_config(&file.page_size),
            margin_mm: file.margin_mm,
            toc: file.toc,
            page_numbers: file.page_numbers,
        }
    }
}

/// [export.docx] table: last-used DOCX export options.
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct DocxExportFile {
    /// "a4" | "letter" | "legal"; unknown values fall back to A4.
    page_size: String,
    #[serde(deserialize_with = "deserialize_bool_or_false")]
    toc: bool,
    /// "embed" | "text-fallback"; unknown values fall back to embed.
    image_policy: String,
}

impl Default for DocxExportFile {
    fn default() -> Self {
        let defaults = DocxExportOptions::default();
        Self {
            page_size: defaults.page_size.config_value().to_string(),
            toc: defaults.toc,
            image_policy: defaults.image_policy.config_value().to_string(),
        }
    }
}

impl From<&DocxExportOptions> for DocxExportFile {
    fn from(options: &DocxExportOptions) -> Self {
        Self {
            page_size: options.page_size.config_value().to_string(),
            toc: options.toc,
            image_policy: options.image_policy.config_value().to_string(),
        }
    }
}

impl From<DocxExportFile> for DocxExportOptions {
    fn from(file: DocxExportFile) -> Self {
        Self {
            page_size: DocxPageSize::from_config(&file.page_size),
            toc: file.toc,
            image_policy: DocxImagePolicy::from_config(&file.image_policy),
        }
    }
}

impl Default for ExportFile {
    fn default() -> Self {
        let defaults = ExportPreferences::default();
        Self {
            backend: defaults.backend.config_value().to_string(),
            pdf_engine: defaults.pdf_engine,
            pandoc_path: defaults.pandoc_path,
            reference_doc: defaults.reference_doc,
            pdf_mainfont: defaults.pdf_mainfont,
            pdf_cjk_font: defaults.pdf_cjk_font,
            pdf: PdfExportFile::default(),
            docx: DocxExportFile::default(),
        }
    }
}

impl Default for PreferencesFile {
    fn default() -> Self {
        Self::from(&AppPreferences::default())
    }
}

impl Default for AutoSaveFile {
    fn default() -> Self {
        let defaults = AutoSavePreferences::default();
        Self {
            enabled: defaults.enabled,
            silent_save: defaults.silent_save,
            delay_secs: defaults.delay_secs,
        }
    }
}

impl From<&AppPreferences> for PreferencesFile {
    fn from(preferences: &AppPreferences) -> Self {
        Self {
            theme: preferences.theme.clone(),
            custom_theme: preferences.custom_theme.clone(),
            language: preferences.language.clone(),
            check_for_updates_on_startup: preferences.check_for_updates_on_startup,
            last_update_check: preferences.last_update_check.clone(),
            focus_mode: preferences.focus_mode,
            typewriter_mode: preferences.typewriter_mode,
            code_line_numbers: preferences.code_line_numbers,
            code_theme: preferences.code_theme.config_value().to_string(),
            code_long_line_wrap: preferences.code_long_line_wrap,
            code_font_size: preferences
                .code_font_size
                .map(|size| normalize_code_font_size(size as i64)),
            preview_adaptive_width: preferences.preview_adaptive_width,
            editor_font_size: normalize_editor_font_size(preferences.editor_font_size as i64),
            rendered_font_size: normalize_rendered_font_size(preferences.rendered_font_size as i64),
            paragraph_spacing: normalize_paragraph_spacing(preferences.paragraph_spacing as i64),
            editor_font_family: crate::model::normalize_font_family(
                preferences.editor_font_family.as_deref(),
            ),
            rendered_font_family: crate::model::normalize_font_family(
                preferences.rendered_font_family.as_deref(),
            ),
            code_font_family: crate::model::normalize_font_family(
                preferences.code_font_family.as_deref(),
            ),
            heading_menu_max_level: preferences.heading_menu_max_level,
            sync_scroll: preferences.sync_scroll,
            show_hidden_files: preferences.show_hidden_files,
            open_in_current_tab: preferences.open_in_current_tab,
            sidebar_visible: preferences.sidebar_visible,
            sidebar_tab: match preferences.sidebar_tab {
                SidebarTab::Files => "files".to_string(),
                SidebarTab::Outline => "outline".to_string(),
            },
            auto_save: AutoSaveFile {
                enabled: preferences.auto_save.enabled,
                silent_save: preferences.auto_save.silent_save,
                delay_secs: preferences.auto_save.delay_secs,
            },
            export: ExportFile {
                backend: preferences.export.backend.config_value().to_string(),
                pdf_engine: preferences.export.pdf_engine.clone(),
                pandoc_path: preferences.export.pandoc_path.clone(),
                reference_doc: preferences.export.reference_doc.clone(),
                pdf_mainfont: preferences.export.pdf_mainfont.clone(),
                pdf_cjk_font: preferences.export.pdf_cjk_font.clone(),
                pdf: PdfExportFile::from(&preferences.export.pdf),
                docx: DocxExportFile::from(&preferences.export.docx),
            },
            shortcuts: preferences.shortcut_overrides.clone(),
        }
    }
}

impl From<PreferencesFile> for AppPreferences {
    fn from(file: PreferencesFile) -> Self {
        Self {
            theme: file.theme,
            custom_theme: file.custom_theme.filter(|name| !name.is_empty()),
            language: file.language,
            check_for_updates_on_startup: file.check_for_updates_on_startup,
            last_update_check: file.last_update_check,
            focus_mode: file.focus_mode,
            typewriter_mode: file.typewriter_mode,
            code_line_numbers: file.code_line_numbers,
            code_theme: CodeTheme::from_config(&file.code_theme),
            code_long_line_wrap: file.code_long_line_wrap,
            code_font_size: file
                .code_font_size
                .map(|size| normalize_code_font_size(size as i64)),
            preview_adaptive_width: file.preview_adaptive_width,
            editor_font_size: normalize_editor_font_size(file.editor_font_size as i64),
            rendered_font_size: normalize_rendered_font_size(file.rendered_font_size as i64),
            paragraph_spacing: normalize_paragraph_spacing(file.paragraph_spacing as i64),
            editor_font_family: crate::model::normalize_font_family(
                file.editor_font_family.as_deref(),
            ),
            rendered_font_family: crate::model::normalize_font_family(
                file.rendered_font_family.as_deref(),
            ),
            code_font_family: crate::model::normalize_font_family(file.code_font_family.as_deref()),
            heading_menu_max_level: normalize_heading_menu_max_level(file.heading_menu_max_level),
            sync_scroll: file.sync_scroll,
            show_hidden_files: file.show_hidden_files,
            open_in_current_tab: file.open_in_current_tab,
            sidebar_visible: file.sidebar_visible,
            sidebar_tab: match file.sidebar_tab.to_ascii_lowercase().as_str() {
                "outline" => SidebarTab::Outline,
                _ => SidebarTab::Files,
            },
            auto_save: AutoSavePreferences {
                enabled: file.auto_save.enabled,
                silent_save: file.auto_save.silent_save,
                delay_secs: crate::model::normalize_auto_save_delay_secs(
                    file.auto_save.delay_secs as i64,
                ),
            },
            export: ExportPreferences {
                backend: ExportBackendPreference::from_config(&file.export.backend),
                pdf_engine: {
                    let engine = file.export.pdf_engine.trim().to_string();
                    if engine.is_empty() {
                        ExportPreferences::default().pdf_engine
                    } else {
                        engine
                    }
                },
                pandoc_path: file.export.pandoc_path,
                reference_doc: file.export.reference_doc,
                pdf_mainfont: file.export.pdf_mainfont,
                pdf_cjk_font: file.export.pdf_cjk_font,
                pdf: file.export.pdf.into(),
                docx: file.export.docx.into(),
            },
            shortcut_overrides: file.shortcuts,
        }
    }
}

/// Loads preferences from the TOML file at `path`. When the file does not
/// exist but a legacy `preferences.conf` sits next to it, the legacy values
/// are migrated: parsed, written out as TOML to `path`, and returned. The
/// legacy file is left in place and ignored on subsequent loads.
pub fn load_app_preferences(path: impl AsRef<Path>) -> io::Result<AppPreferences> {
    let path = path.as_ref();
    if path.exists() {
        return parse_app_preferences(&fs::read_to_string(path)?);
    }

    if let Some(legacy_path) = path
        .parent()
        .map(|dir| dir.join(LEGACY_PREFERENCES_FILE_NAME))
        .filter(|candidate| candidate.exists())
    {
        let preferences = parse_legacy_app_preferences(&fs::read_to_string(&legacy_path)?)?;
        save_app_preferences(path, &preferences)?;
        tracing::info!(
            legacy = %legacy_path.display(),
            config = %path.display(),
            "migrated legacy preferences to TOML"
        );
        return Ok(preferences);
    }

    Ok(AppPreferences::default())
}

pub fn save_app_preferences(
    path: impl AsRef<Path>,
    preferences: &AppPreferences,
) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    super::atomic_write(path, render_app_preferences(preferences).as_bytes())
}

/// Parses the TOML preferences format. Missing fields take their defaults.
pub fn parse_app_preferences(text: &str) -> io::Result<AppPreferences> {
    let file: PreferencesFile = toml::from_str(text)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(file.into())
}

/// Renders preferences as TOML (the on-disk `config.toml` format).
pub fn render_app_preferences(preferences: &AppPreferences) -> String {
    toml::to_string_pretty(&PreferencesFile::from(preferences))
        .expect("preferences serialize to TOML")
}

/// Parses the retired `key=value` format. Kept only as the migration reader
/// for pre-TOML installations.
pub fn parse_legacy_app_preferences(text: &str) -> io::Result<AppPreferences> {
    let mut preferences = AppPreferences::default();
    for (line_index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid preference line {}", line_index + 1),
            ));
        };
        match key.trim() {
            "theme" => preferences.theme = value.trim().to_string(),
            "custom_theme" => {
                let value = value.trim();
                preferences.custom_theme = (!value.is_empty()).then(|| value.to_string());
            }
            "focus_mode" => preferences.focus_mode = parse_preference_bool(value.trim())?,
            "typewriter_mode" => {
                preferences.typewriter_mode = parse_preference_bool(value.trim())?;
            }
            "code_line_numbers" => {
                preferences.code_line_numbers = parse_preference_bool(value.trim())?;
            }
            "preview_adaptive_width" => {
                preferences.preview_adaptive_width = parse_preference_bool(value.trim())?;
            }
            "sync_scroll" => {
                preferences.sync_scroll = parse_preference_bool(value.trim())?;
            }
            "sidebar_visible" => {
                preferences.sidebar_visible = parse_preference_bool(value.trim())?;
            }
            "sidebar_tab" => {
                preferences.sidebar_tab = match value.trim().to_ascii_lowercase().as_str() {
                    "outline" => SidebarTab::Outline,
                    // Unknown / missing values fall back to Files.
                    _ => SidebarTab::Files,
                };
            }
            "language" => {
                let value = value.trim();
                if !value.is_empty() {
                    preferences.language = value.to_string();
                }
            }
            _ => {}
        }
    }
    Ok(preferences)
}

pub(crate) fn parse_preference_bool(value: &str) -> io::Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid boolean preference value '{value}'"),
        )),
    }
}

fn default_heading_menu_max_level() -> u8 {
    AppPreferences::default().heading_menu_max_level
}

fn deserialize_bool_or_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(value.as_bool().unwrap_or(false))
}

fn deserialize_bool_or_true<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(value.as_bool().unwrap_or(true))
}

fn deserialize_pdf_margin_mm<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let parsed =
        deserialize_integer_or(deserializer, i64::from(crate::model::DEFAULT_PDF_MARGIN_MM))?;
    Ok(u32::try_from(parsed).unwrap_or(crate::model::DEFAULT_PDF_MARGIN_MM))
}

fn deserialize_editor_font_size<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(normalize_editor_font_size(deserialize_integer_or(
        deserializer,
        DEFAULT_EDITOR_FONT_SIZE as i64,
    )?))
}

fn deserialize_rendered_font_size<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(normalize_rendered_font_size(deserialize_integer_or(
        deserializer,
        DEFAULT_RENDERED_FONT_SIZE as i64,
    )?))
}

fn deserialize_paragraph_spacing<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(normalize_paragraph_spacing(deserialize_integer_or(
        deserializer,
        DEFAULT_PARAGRAPH_SPACING as i64,
    )?))
}

/// Explicit code font size: an integer is clamped to the supported range;
/// any other value type (or absence) means "follow the reading size".
fn deserialize_optional_code_font_size<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(value
        .as_integer()
        .map(|size| normalize_code_font_size(size)))
}

fn deserialize_integer_or<'de, D>(deserializer: D, default: i64) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(value.as_integer().unwrap_or(default))
}

/// Font-family preferences are free-form strings: a string value is kept
/// (trimmed; empty treated as unset), any other value type degrades to unset
/// rather than blocking startup.
fn deserialize_optional_font_family<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(crate::model::normalize_font_family(value.as_str()))
}

/// Path-like config strings (`export.pandoc_path`, `export.reference_doc`):
/// a string value is kept (trimmed; empty treated as unset), any other value
/// type degrades to unset rather than blocking startup.
fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(crate::model::normalize_font_family(value.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_overrides_round_trip_and_empty_table_is_omitted() {
        let defaults = render_app_preferences(&AppPreferences::default());
        assert!(
            !defaults.contains("[shortcuts]"),
            "empty shortcut overrides must not create a TOML table: {defaults}"
        );

        let mut preferences = AppPreferences::default();
        preferences
            .shortcut_overrides
            .insert("bold".to_string(), "ctrl-alt-b".to_string());
        preferences
            .shortcut_overrides
            .insert("show-shortcuts".to_string(), "f9".to_string());

        let rendered = render_app_preferences(&preferences);
        assert!(rendered.contains("[shortcuts]"));
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert_eq!(parsed.shortcut_overrides, preferences.shortcut_overrides);
    }

    #[test]
    fn typography_preferences_round_trip_and_default_when_missing() {
        let preferences = AppPreferences {
            editor_font_size: 18,
            rendered_font_size: 20,
            paragraph_spacing: 16,
            ..AppPreferences::default()
        };
        let rendered = render_app_preferences(&preferences);
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert_eq!(parsed.editor_font_size, 18);
        assert_eq!(parsed.rendered_font_size, 20);
        assert_eq!(parsed.paragraph_spacing, 16);

        let missing = parse_app_preferences("theme = \"Paper\"\n").unwrap();
        assert_eq!(missing.editor_font_size, DEFAULT_EDITOR_FONT_SIZE);
        assert_eq!(missing.rendered_font_size, DEFAULT_RENDERED_FONT_SIZE);
        assert_eq!(missing.paragraph_spacing, DEFAULT_PARAGRAPH_SPACING);
    }

    #[test]
    fn typography_preferences_default_invalid_types_and_clamp_numbers() {
        let invalid = parse_app_preferences(
            "editor_font_size = \"large\"\nrendered_font_size = false\nparagraph_spacing = []\n",
        )
        .unwrap();
        assert_eq!(invalid.editor_font_size, DEFAULT_EDITOR_FONT_SIZE);
        assert_eq!(invalid.rendered_font_size, DEFAULT_RENDERED_FONT_SIZE);
        assert_eq!(invalid.paragraph_spacing, DEFAULT_PARAGRAPH_SPACING);

        let bounded = parse_app_preferences(
            "editor_font_size = -5\nrendered_font_size = 1000\nparagraph_spacing = 1000\n",
        )
        .unwrap();
        assert_eq!(bounded.editor_font_size, crate::model::MIN_EDITOR_FONT_SIZE);
        assert_eq!(
            bounded.rendered_font_size,
            crate::model::MAX_RENDERED_FONT_SIZE
        );
        assert_eq!(
            bounded.paragraph_spacing,
            crate::model::MAX_PARAGRAPH_SPACING
        );
    }

    #[test]
    fn typography_normalizers_apply_documented_bounds() {
        assert_eq!(
            normalize_editor_font_size(i64::MIN),
            crate::model::MIN_EDITOR_FONT_SIZE
        );
        assert_eq!(
            normalize_editor_font_size(i64::MAX),
            crate::model::MAX_EDITOR_FONT_SIZE
        );
        assert_eq!(
            normalize_rendered_font_size(i64::MIN),
            crate::model::MIN_RENDERED_FONT_SIZE
        );
        assert_eq!(
            normalize_rendered_font_size(i64::MAX),
            crate::model::MAX_RENDERED_FONT_SIZE
        );
        assert_eq!(normalize_paragraph_spacing(i64::MIN), 0);
        assert_eq!(
            normalize_paragraph_spacing(i64::MAX),
            crate::model::MAX_PARAGRAPH_SPACING
        );
    }

    #[test]
    fn sync_scroll_defaults_to_false() {
        assert!(!AppPreferences::default().sync_scroll);
    }

    #[test]
    fn code_display_preferences_round_trip() {
        let preferences = AppPreferences {
            code_theme: CodeTheme::Light,
            code_long_line_wrap: false,
            code_font_size: Some(16),
            ..AppPreferences::default()
        };
        let rendered = render_app_preferences(&preferences);
        assert!(rendered.contains("code_theme = \"light\""));
        assert!(rendered.contains("code_long_line_wrap = false"));
        assert!(rendered.contains("code_font_size = 16"));
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert_eq!(parsed.code_theme, CodeTheme::Light);
        assert!(!parsed.code_long_line_wrap);
        assert_eq!(parsed.code_font_size, Some(16));

        // The default (dark / wrapped / follow reading size) omits the size key.
        let defaults = render_app_preferences(&AppPreferences::default());
        assert!(defaults.contains("code_theme = \"dark\""));
        assert!(defaults.contains("code_long_line_wrap = true"));
        assert!(!defaults.contains("code_font_size"));
    }

    #[test]
    fn code_display_preferences_default_when_missing() {
        let missing = parse_app_preferences("theme = \"Paper\"\n").unwrap();
        assert_eq!(missing.code_theme, CodeTheme::Dark);
        assert!(missing.code_long_line_wrap);
        assert_eq!(missing.code_font_size, None);
    }

    #[test]
    fn code_display_preferences_invalid_values_are_safe() {
        let invalid = parse_app_preferences(
            "code_theme = \"solarized\"\ncode_long_line_wrap = \"maybe\"\ncode_font_size = \"big\"\n",
        )
        .unwrap();
        assert_eq!(invalid.code_theme, CodeTheme::Dark);
        assert!(invalid.code_long_line_wrap);
        assert_eq!(invalid.code_font_size, None);

        let bounded = parse_app_preferences("code_font_size = 999\n").unwrap();
        assert_eq!(
            bounded.code_font_size,
            Some(crate::model::MAX_CODE_FONT_SIZE)
        );
        let floored = parse_app_preferences("code_font_size = 1\n").unwrap();
        assert_eq!(
            floored.code_font_size,
            Some(crate::model::MIN_CODE_FONT_SIZE)
        );
    }

    #[test]
    fn font_family_preferences_round_trip_verbatim() {
        let preferences = AppPreferences {
            editor_font_family: Some("Cascadia Code".to_string()),
            rendered_font_family: Some("Source Serif 4".to_string()),
            code_font_family: Some(".SystemUIFont".to_string()),
            ..AppPreferences::default()
        };
        let rendered = render_app_preferences(&preferences);
        assert!(rendered.contains("editor_font_family = \"Cascadia Code\""));
        assert!(rendered.contains("rendered_font_family = \"Source Serif 4\""));
        assert!(rendered.contains("code_font_family = \".SystemUIFont\""));
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert_eq!(parsed.editor_font_family.as_deref(), Some("Cascadia Code"));
        assert_eq!(
            parsed.rendered_font_family.as_deref(),
            Some("Source Serif 4")
        );
        assert_eq!(parsed.code_font_family.as_deref(), Some(".SystemUIFont"));
    }

    #[test]
    fn font_family_preferences_absent_empty_and_invalid_are_none() {
        // Absent keys, empty/whitespace strings, and non-string values all
        // mean "follow theme/default" and never block loading.
        assert!(AppPreferences::default().editor_font_family.is_none());
        assert!(AppPreferences::default().rendered_font_family.is_none());
        assert!(AppPreferences::default().code_font_family.is_none());

        let text = "theme = \"Paper\"\neditor_font_family = \"\"\nrendered_font_family = \"   \"\ncode_font_family = 12\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert!(parsed.editor_font_family.is_none());
        assert!(parsed.rendered_font_family.is_none());
        assert!(parsed.code_font_family.is_none());
    }

    #[test]
    fn font_family_preferences_omit_keys_when_unset() {
        let rendered = render_app_preferences(&AppPreferences::default());
        assert!(!rendered.contains("font_family"));
    }

    #[test]
    fn sync_scroll_round_trips_through_toml() {
        let preferences = AppPreferences {
            sync_scroll: true,
            ..AppPreferences::default()
        };
        let rendered = render_app_preferences(&preferences);
        assert!(
            rendered.contains("sync_scroll = true"),
            "rendered TOML should set sync_scroll = true: {rendered}"
        );
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert!(parsed.sync_scroll, "parsed sync_scroll should be true");
    }

    #[test]
    fn update_check_preferences_default_off_and_none() {
        let defaults = AppPreferences::default();
        assert!(!defaults.check_for_updates_on_startup);
        assert!(defaults.last_update_check.is_none());
    }

    #[test]
    fn update_check_preferences_round_trip_through_toml() {
        let preferences = AppPreferences {
            check_for_updates_on_startup: true,
            last_update_check: Some("2026-07-27T10:30:00Z".to_string()),
            ..AppPreferences::default()
        };
        let rendered = render_app_preferences(&preferences);
        assert!(
            rendered.contains("check_for_updates_on_startup = true"),
            "rendered TOML should set check_for_updates_on_startup = true: {rendered}"
        );
        assert!(
            rendered.contains("last_update_check = \"2026-07-27T10:30:00Z\""),
            "rendered TOML should set last_update_check: {rendered}"
        );
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert!(parsed.check_for_updates_on_startup);
        assert_eq!(
            parsed.last_update_check.as_deref(),
            Some("2026-07-27T10:30:00Z"),
        );
    }

    #[test]
    fn missing_update_check_preferences_fall_back_to_defaults() {
        // A pre-existing config.toml written before these preferences existed
        // omits both fields; the deserializer must treat them as defaults.
        let text = "theme = \"Paper\"\nlanguage = \"en\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert!(!parsed.check_for_updates_on_startup);
        assert!(parsed.last_update_check.is_none());
    }

    #[test]
    fn missing_sync_scroll_falls_back_to_false() {
        // A pre-existing config.toml written before this preference existed
        // omits the field entirely; the deserializer must treat it as false.
        let text = "theme = \"Paper\"\nlanguage = \"en\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert!(!parsed.sync_scroll);
    }

    #[test]
    fn invalid_sync_scroll_value_falls_back_to_false() {
        // A corrupt/unknown value must not abort loading; it degrades to false.
        let text = "theme = \"Paper\"\nsync_scroll = \"yes\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert!(!parsed.sync_scroll);
    }

    #[test]
    fn show_hidden_files_defaults_to_false() {
        assert!(!AppPreferences::default().show_hidden_files);
    }

    #[test]
    fn show_hidden_files_round_trips_through_toml() {
        let preferences = AppPreferences {
            show_hidden_files: true,
            ..AppPreferences::default()
        };
        let rendered = render_app_preferences(&preferences);
        assert!(
            rendered.contains("show_hidden_files = true"),
            "rendered TOML should set show_hidden_files = true: {rendered}"
        );
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert!(
            parsed.show_hidden_files,
            "parsed show_hidden_files should be true"
        );
    }

    #[test]
    fn missing_show_hidden_files_falls_back_to_false() {
        // A pre-existing config.toml written before this preference existed
        // omits the field entirely; the deserializer must treat it as false.
        let text = "theme = \"Paper\"\nlanguage = \"en\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert!(!parsed.show_hidden_files);
    }

    #[test]
    fn invalid_show_hidden_files_value_falls_back_to_false() {
        // A corrupt/unknown value must not abort loading; it degrades to false.
        let text = "theme = \"Paper\"\nshow_hidden_files = \"yes\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert!(!parsed.show_hidden_files);
    }

    #[test]
    fn open_in_current_tab_defaults_to_true() {
        assert!(AppPreferences::default().open_in_current_tab);
    }

    #[test]
    fn open_in_current_tab_round_trips_through_toml() {
        let preferences = AppPreferences {
            open_in_current_tab: false,
            ..AppPreferences::default()
        };
        let rendered = render_app_preferences(&preferences);
        assert!(
            rendered.contains("open_in_current_tab = false"),
            "rendered TOML should set open_in_current_tab = false: {rendered}"
        );
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert!(
            !parsed.open_in_current_tab,
            "parsed open_in_current_tab should be false"
        );
    }

    #[test]
    fn missing_open_in_current_tab_falls_back_to_true() {
        // A pre-existing config.toml written before this preference existed
        // omits the field entirely; the deserializer must treat it as true.
        let text = "theme = \"Paper\"\nlanguage = \"en\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert!(parsed.open_in_current_tab);
    }

    #[test]
    fn invalid_open_in_current_tab_value_falls_back_to_true() {
        // A corrupt/unknown value must not abort loading; it degrades to the
        // default (on).
        let text = "theme = \"Paper\"\nopen_in_current_tab = \"yes\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert!(parsed.open_in_current_tab);
    }

    #[test]
    fn legacy_config_migrates_sync_scroll() {
        let text = "theme = Paper\npreview_adaptive_width = true\nsync_scroll = true\n";
        let parsed = parse_legacy_app_preferences(text).unwrap();
        assert!(parsed.sync_scroll);
        assert!(parsed.preview_adaptive_width);

        // And a legacy file without the field keeps the default.
        let parsed_without = parse_legacy_app_preferences("theme = Paper\n").unwrap();
        assert!(!parsed_without.sync_scroll);
    }

    #[test]
    fn export_backend_defaults_to_builtin_and_parses_from_config() {
        // Pre-existing config.toml without the key keeps the built-in default.
        let parsed = parse_app_preferences("[export]\npdf_engine = \"pdfroff\"\n").unwrap();
        assert_eq!(
            parsed.export.backend,
            crate::model::ExportBackendPreference::BuiltIn
        );

        let parsed = parse_app_preferences("[export]\nbackend = \"pandoc\"\n").unwrap();
        assert_eq!(
            parsed.export.backend,
            crate::model::ExportBackendPreference::Pandoc
        );

        // Unknown tokens fall back to the built-in default.
        let parsed = parse_app_preferences("[export]\nbackend = \"typst\"\n").unwrap();
        assert_eq!(
            parsed.export.backend,
            crate::model::ExportBackendPreference::BuiltIn
        );
    }

    #[test]
    fn export_backend_round_trips_through_config() {
        let mut preferences = AppPreferences::default();
        preferences.export.backend = crate::model::ExportBackendPreference::Pandoc;
        let rendered = render_app_preferences(&preferences);
        assert!(rendered.contains("backend = \"pandoc\""));
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert_eq!(
            parsed.export.backend,
            crate::model::ExportBackendPreference::Pandoc
        );
    }

    #[test]
    fn export_paths_parse_from_config() {
        let text = "[export]\npdf_engine = \"tectonic\"\npandoc_path = \"/opt/pandoc/bin/pandoc\"\nreference_doc = \"templates/my-reference.docx\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert_eq!(parsed.export.pdf_engine, "tectonic");
        assert_eq!(
            parsed.export.pandoc_path.as_deref(),
            Some("/opt/pandoc/bin/pandoc")
        );
        assert_eq!(
            parsed.export.reference_doc.as_deref(),
            Some("templates/my-reference.docx")
        );
    }

    #[test]
    fn export_paths_default_to_none_when_missing_or_blank() {
        // Pre-existing config.toml without the keys keeps the defaults.
        let parsed = parse_app_preferences("[export]\npdf_engine = \"pdfroff\"\n").unwrap();
        assert_eq!(parsed.export.pdf_engine, "pdfroff");
        assert!(parsed.export.pandoc_path.is_none());
        assert!(parsed.export.reference_doc.is_none());

        // Blank or wrongly-typed values degrade to unset.
        let parsed =
            parse_app_preferences("[export]\npandoc_path = \"   \"\nreference_doc = 12\n").unwrap();
        assert!(parsed.export.pandoc_path.is_none());
        assert!(parsed.export.reference_doc.is_none());
    }

    #[test]
    fn export_paths_round_trip_and_omit_when_unset() {
        let defaults = render_app_preferences(&AppPreferences::default());
        assert!(!defaults.contains("pandoc_path"));
        assert!(!defaults.contains("reference_doc"));

        let mut preferences = AppPreferences::default();
        preferences.export.pandoc_path = Some("C:\\tools\\pandoc.exe".to_string());
        preferences.export.reference_doc = Some("ref.docx".to_string());
        let rendered = render_app_preferences(&preferences);
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert_eq!(
            parsed.export.pandoc_path.as_deref(),
            Some("C:\\tools\\pandoc.exe")
        );
        assert_eq!(parsed.export.reference_doc.as_deref(), Some("ref.docx"));
    }

    #[test]
    fn docx_export_options_default_to_current_behavior() {
        let defaults = AppPreferences::default().export.docx;
        assert_eq!(defaults.page_size, DocxPageSize::A4);
        assert!(!defaults.toc);
        assert_eq!(defaults.image_policy, DocxImagePolicy::Embed);

        // A pre-existing config.toml without [export.docx] keeps the defaults.
        let parsed = parse_app_preferences("[export]\npdf_engine = \"pdfroff\"\n").unwrap();
        assert_eq!(parsed.export.docx, DocxExportOptions::default());
    }

    #[test]
    fn docx_export_options_parse_from_config() {
        let text =
            "[export.docx]\npage_size = \"letter\"\ntoc = true\nimage_policy = \"text-fallback\"\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert_eq!(parsed.export.docx.page_size, DocxPageSize::Letter);
        assert!(parsed.export.docx.toc);
        assert_eq!(
            parsed.export.docx.image_policy,
            DocxImagePolicy::TextFallback
        );
    }

    #[test]
    fn docx_export_options_tolerate_unknown_values() {
        let text = "[export.docx]\npage_size = \"tabloid\"\ntoc = \"yes\"\nimage_policy = 12\n";
        // Non-string image_policy would fail deserialization, so use a string
        // with an unknown token instead.
        let text = text.replace("image_policy = 12", "image_policy = \"mystery\"");
        let parsed = parse_app_preferences(&text).unwrap();
        assert_eq!(parsed.export.docx.page_size, DocxPageSize::A4);
        assert!(!parsed.export.docx.toc);
        assert_eq!(parsed.export.docx.image_policy, DocxImagePolicy::Embed);
    }

    #[test]
    fn docx_export_options_round_trip() {
        let mut preferences = AppPreferences::default();
        preferences.export.docx = DocxExportOptions {
            page_size: DocxPageSize::Legal,
            toc: true,
            image_policy: DocxImagePolicy::TextFallback,
        };
        let rendered = render_app_preferences(&preferences);
        assert!(rendered.contains("[export.docx]"));
        assert!(rendered.contains("page_size = \"legal\""));
        assert!(rendered.contains("toc = true"));
        assert!(rendered.contains("image_policy = \"text-fallback\""));
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert_eq!(parsed.export.docx, preferences.export.docx);
    }

    #[test]
    fn pdf_export_options_default_to_current_behavior() {
        let defaults = AppPreferences::default().export.pdf;
        assert_eq!(defaults.page_size, PdfPageSize::A4);
        assert_eq!(defaults.margin_mm, crate::model::DEFAULT_PDF_MARGIN_MM);
        assert!(!defaults.toc);
        assert!(defaults.page_numbers);

        // A pre-existing config.toml without [export.pdf] keeps the defaults.
        let parsed = parse_app_preferences("[export]\npdf_engine = \"pdfroff\"\n").unwrap();
        assert_eq!(parsed.export.pdf, PdfExportOptions::default());
        assert!(parsed.export.pdf_mainfont.is_none());
        assert!(parsed.export.pdf_cjk_font.is_none());
    }

    #[test]
    fn pdf_export_options_parse_from_config() {
        let text = "[export.pdf]\npage_size = \"letter\"\nmargin_mm = 20\ntoc = true\npage_numbers = false\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert_eq!(parsed.export.pdf.page_size, PdfPageSize::Letter);
        assert_eq!(parsed.export.pdf.margin_mm, 20);
        assert!(parsed.export.pdf.toc);
        assert!(!parsed.export.pdf.page_numbers);

        let fonts =
            "[export]\npdf_mainfont = \"Source Serif 4\"\npdf_cjk_font = \"Source Han Sans SC\"\n";
        let parsed = parse_app_preferences(fonts).unwrap();
        assert_eq!(
            parsed.export.pdf_mainfont.as_deref(),
            Some("Source Serif 4")
        );
        assert_eq!(
            parsed.export.pdf_cjk_font.as_deref(),
            Some("Source Han Sans SC")
        );
    }

    #[test]
    fn pdf_export_options_tolerate_unknown_values() {
        let text = "[export.pdf]\npage_size = \"tabloid\"\nmargin_mm = \"wide\"\ntoc = \"yes\"\npage_numbers = 12\n";
        let parsed = parse_app_preferences(text).unwrap();
        assert_eq!(parsed.export.pdf.page_size, PdfPageSize::A4);
        assert_eq!(
            parsed.export.pdf.margin_mm,
            crate::model::DEFAULT_PDF_MARGIN_MM
        );
        assert!(!parsed.export.pdf.toc);
        assert!(parsed.export.pdf.page_numbers);

        // Negative margins degrade to the default.
        let parsed = parse_app_preferences("[export.pdf]\nmargin_mm = -5\n").unwrap();
        assert_eq!(
            parsed.export.pdf.margin_mm,
            crate::model::DEFAULT_PDF_MARGIN_MM
        );
    }

    #[test]
    fn pdf_export_options_round_trip() {
        let mut preferences = AppPreferences::default();
        preferences.export.pdf = PdfExportOptions {
            page_size: PdfPageSize::Legal,
            margin_mm: 18,
            toc: true,
            page_numbers: false,
        };
        let rendered = render_app_preferences(&preferences);
        assert!(rendered.contains("[export.pdf]"));
        assert!(rendered.contains("page_size = \"legal\""));
        assert!(rendered.contains("margin_mm = 18"));
        assert!(rendered.contains("toc = true"));
        assert!(rendered.contains("page_numbers = false"));
        let parsed = parse_app_preferences(&rendered).unwrap();
        assert_eq!(parsed.export.pdf, preferences.export.pdf);
    }
}
