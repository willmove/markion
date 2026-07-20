//! App preference parsing, rendering, and persistence.
//!
//! Preferences persist as TOML (`config.toml`), a design adopted from
//! Typune's filesystem crate: every field is optional and defaulted, and
//! auto-save behavior lives in an `[auto_save]` table. The retired
//! hand-written `key=value` format (`preferences.conf`) is still readable so
//! `load_app_preferences` can migrate it to TOML once, after which the legacy
//! file is ignored.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::model::{
    AppPreferences, AutoSavePreferences, DEFAULT_EDITOR_FONT_SIZE, DEFAULT_PARAGRAPH_SPACING,
    DEFAULT_RENDERED_FONT_SIZE, ExportPreferences, SidebarTab, normalize_editor_font_size,
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
    focus_mode: bool,
    typewriter_mode: bool,
    code_line_numbers: bool,
    #[serde(deserialize_with = "deserialize_bool_or_false")]
    preview_adaptive_width: bool,
    #[serde(deserialize_with = "deserialize_editor_font_size")]
    editor_font_size: u16,
    #[serde(deserialize_with = "deserialize_rendered_font_size")]
    rendered_font_size: u16,
    #[serde(deserialize_with = "deserialize_paragraph_spacing")]
    paragraph_spacing: u16,
    #[serde(default = "default_heading_menu_max_level")]
    heading_menu_max_level: u8,
    #[serde(deserialize_with = "deserialize_bool_or_false")]
    sync_scroll: bool,
    sidebar_visible: bool,
    /// "files" or "outline"; unknown values fall back to Files like the
    /// legacy format did.
    sidebar_tab: String,
    auto_save: AutoSaveFile,
    export: ExportFile,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct AutoSaveFile {
    enabled: bool,
    delay_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct ExportFile {
    pdf_engine: String,
}

impl Default for ExportFile {
    fn default() -> Self {
        Self {
            pdf_engine: ExportPreferences::default().pdf_engine,
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
            focus_mode: preferences.focus_mode,
            typewriter_mode: preferences.typewriter_mode,
            code_line_numbers: preferences.code_line_numbers,
            preview_adaptive_width: preferences.preview_adaptive_width,
            editor_font_size: normalize_editor_font_size(preferences.editor_font_size as i64),
            rendered_font_size: normalize_rendered_font_size(preferences.rendered_font_size as i64),
            paragraph_spacing: normalize_paragraph_spacing(preferences.paragraph_spacing as i64),
            heading_menu_max_level: preferences.heading_menu_max_level,
            sync_scroll: preferences.sync_scroll,
            sidebar_visible: preferences.sidebar_visible,
            sidebar_tab: match preferences.sidebar_tab {
                SidebarTab::Files => "files".to_string(),
                SidebarTab::Outline => "outline".to_string(),
            },
            auto_save: AutoSaveFile {
                enabled: preferences.auto_save.enabled,
                delay_secs: preferences.auto_save.delay_secs,
            },
            export: ExportFile {
                pdf_engine: preferences.export.pdf_engine.clone(),
            },
        }
    }
}

impl From<PreferencesFile> for AppPreferences {
    fn from(file: PreferencesFile) -> Self {
        Self {
            theme: file.theme,
            custom_theme: file.custom_theme.filter(|name| !name.is_empty()),
            language: file.language,
            focus_mode: file.focus_mode,
            typewriter_mode: file.typewriter_mode,
            code_line_numbers: file.code_line_numbers,
            preview_adaptive_width: file.preview_adaptive_width,
            editor_font_size: normalize_editor_font_size(file.editor_font_size as i64),
            rendered_font_size: normalize_rendered_font_size(file.rendered_font_size as i64),
            paragraph_spacing: normalize_paragraph_spacing(file.paragraph_spacing as i64),
            heading_menu_max_level: normalize_heading_menu_max_level(file.heading_menu_max_level),
            sync_scroll: file.sync_scroll,
            sidebar_visible: file.sidebar_visible,
            sidebar_tab: match file.sidebar_tab.to_ascii_lowercase().as_str() {
                "outline" => SidebarTab::Outline,
                _ => SidebarTab::Files,
            },
            auto_save: AutoSavePreferences {
                enabled: file.auto_save.enabled,
                delay_secs: file.auto_save.delay_secs,
            },
            export: ExportPreferences {
                pdf_engine: {
                    let engine = file.export.pdf_engine.trim().to_string();
                    if engine.is_empty() {
                        ExportPreferences::default().pdf_engine
                    } else {
                        engine
                    }
                },
            },
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
    fs::write(path, render_app_preferences(preferences))
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

fn deserialize_integer_or<'de, D>(deserializer: D, default: i64) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(value.as_integer().unwrap_or(default))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn legacy_config_migrates_sync_scroll() {
        let text = "theme = Paper\npreview_adaptive_width = true\nsync_scroll = true\n";
        let parsed = parse_legacy_app_preferences(text).unwrap();
        assert!(parsed.sync_scroll);
        assert!(parsed.preview_adaptive_width);

        // And a legacy file without the field keeps the default.
        let parsed_without = parse_legacy_app_preferences("theme = Paper\n").unwrap();
        assert!(!parsed_without.sync_scroll);
    }
}
