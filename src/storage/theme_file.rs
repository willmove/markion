//! Custom theme file parsing, rendering, listing, and persistence.
//!
//! Custom themes are authored as TOML (`.toml`) in the themes directory. The
//! retired `.theme` `key=value` format is read once as a migration source:
//! when a `.theme` exists with no `.toml` of the same stem, it is parsed,
//! written out as `.toml`, and left in place (ignored thereafter). This
//! mirrors the `preferences.conf` → `config.toml` migration in
//! `src/storage/preferences.rs`.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::model::{ThemeColors, ThemeDefinition};
use crate::storage::preferences::parse_preference_bool;

/// Fallback palette used when a TOML theme omits a `[colors]` key (or the
/// whole sub-table). Matches the historical `.theme` defaults so a migrated
/// file that only set a few keys behaves the same as before.
fn default_colors() -> ThemeColors {
    ThemeColors {
        app_bg: 0xf8fafc,
        panel_bg: 0xffffff,
        surface_bg: 0xffffff,
        text: 0x0f172a,
        muted: 0x64748b,
        border: 0xdbe4ee,
        active_bg: 0xe0ecff,
        active_text: 0x1d4ed8,
    }
}

// ---------------------------------------------------------------------------
// TOML schema
// ---------------------------------------------------------------------------

/// On-disk TOML representation of a custom theme.
///
/// ```toml
/// name = "Midnight"
/// is_dark = true
///
/// [colors]
/// app_bg = "#10131a"
/// panel_bg = "#171b24"
/// # ... 8 keys
/// ```
#[derive(Serialize, Deserialize)]
struct ThemeFile {
    name: String,
    #[serde(default)]
    is_dark: bool,
    #[serde(default)]
    colors: ThemeColorsFile,
}

/// TOML `[colors]` sub-table. Every key defaults so partial files load.
#[derive(Serialize, Deserialize, Default)]
struct ThemeColorsFile {
    #[serde(default = "default_app_bg", with = "color_opt")]
    app_bg: u32,
    #[serde(default = "default_panel_bg", with = "color_opt")]
    panel_bg: u32,
    #[serde(default = "default_surface_bg", with = "color_opt")]
    surface_bg: u32,
    #[serde(default = "default_text", with = "color_opt")]
    text: u32,
    #[serde(default = "default_muted", with = "color_opt")]
    muted: u32,
    #[serde(default = "default_border", with = "color_opt")]
    border: u32,
    #[serde(default = "default_active_bg", with = "color_opt")]
    active_bg: u32,
    #[serde(default = "default_active_text", with = "color_opt")]
    active_text: u32,
}

// Per-key default functions let `#[serde(default = "...")]` fall back to the
// historical palette value when a key is absent.
fn default_app_bg() -> u32 {
    default_colors().app_bg
}
fn default_panel_bg() -> u32 {
    default_colors().panel_bg
}
fn default_surface_bg() -> u32 {
    default_colors().surface_bg
}
fn default_text() -> u32 {
    default_colors().text
}
fn default_muted() -> u32 {
    default_colors().muted
}
fn default_border() -> u32 {
    default_colors().border
}
fn default_active_bg() -> u32 {
    default_colors().active_bg
}
fn default_active_text() -> u32 {
    default_colors().active_text
}

/// Serde adapter that serializes a `u32` RGB value as a `"#rrggbb"` string and
/// deserializes leniently — accepting either `"#rrggbb"` or bare `"rrggbb"`,
/// and treating a missing field as absent (so the per-key default applies via
/// the outer `Option`).
mod color_opt {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &u32, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("#{:06x}", value))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
        let raw: String = String::deserialize(d)?;
        parse_color_str(&raw).map_err(serde::de::Error::custom)
    }

    fn parse_color_str(raw: &str) -> Result<u32, String> {
        let value = raw.trim().trim_start_matches('#');
        if value.len() != 6 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(format!("invalid color '{raw}'"));
        }
        u32::from_str_radix(value, 16).map_err(|err| err.to_string())
    }
}

impl From<ThemeFile> for ThemeDefinition {
    fn from(file: ThemeFile) -> Self {
        ThemeDefinition {
            name: file.name,
            is_dark: file.is_dark,
            colors: ThemeColors {
                app_bg: file.colors.app_bg,
                panel_bg: file.colors.panel_bg,
                surface_bg: file.colors.surface_bg,
                text: file.colors.text,
                muted: file.colors.muted,
                border: file.colors.border,
                active_bg: file.colors.active_bg,
                active_text: file.colors.active_text,
            },
        }
    }
}

impl From<ThemeDefinition> for ThemeFile {
    fn from(theme: ThemeDefinition) -> Self {
        ThemeFile {
            name: theme.name,
            is_dark: theme.is_dark,
            colors: ThemeColorsFile {
                app_bg: theme.colors.app_bg,
                panel_bg: theme.colors.panel_bg,
                surface_bg: theme.colors.surface_bg,
                text: theme.colors.text,
                muted: theme.colors.muted,
                border: theme.colors.border,
                active_bg: theme.colors.active_bg,
                active_text: theme.colors.active_text,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Loads a custom theme from the `.toml` file at `path`. When `path` (a
/// `.toml`) does not exist but a legacy `.theme` with the same stem sits next
/// to it, the legacy file is parsed, written out as `.toml` to `path`, and
/// returned. The legacy file is left in place and ignored on subsequent loads.
pub fn load_theme_definition(path: impl AsRef<Path>) -> io::Result<ThemeDefinition> {
    let path = path.as_ref();
    if path.exists() {
        return parse_theme_definition(&fs::read_to_string(path)?);
    }

    // Migration: look for a legacy `.theme` of the same stem.
    if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
        let legacy_path = path.with_extension("theme");
        if legacy_path.exists() {
            let theme = parse_legacy_theme_definition(&fs::read_to_string(&legacy_path)?)?;
            save_theme_definition(path, &theme)?;
            tracing::info!(
                legacy = %legacy_path.display(),
                theme = %path.display(),
                "migrated legacy .theme to TOML"
            );
            return Ok(theme);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("theme file not found: {}", path.display()),
    ))
}

pub fn save_theme_definition(path: impl AsRef<Path>, theme: &ThemeDefinition) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, render_theme_definition(theme))
}

/// Lists custom themes in `dir`. Reads `.toml` themes, plus any `.theme` whose
/// stem has no `.toml` beside it (the migration case). Stems are deduped so a
/// migrated pair (`midnight.theme` + `midnight.toml`) surfaces as a single
/// theme, sourced from the `.toml`.
pub fn list_theme_definitions(dir: impl AsRef<Path>) -> io::Result<Vec<ThemeDefinition>> {
    let dir = dir.as_ref();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    // Collect candidate paths: every `.toml` and every `.theme`. Track which
    // stems have a `.toml` so the `.theme` is only read when orphaned.
    let mut toml_stems = std::collections::HashSet::new();
    let mut candidates: Vec<(std::path::PathBuf, bool)> = Vec::new(); // (path, is_toml)
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let ext = path.extension().and_then(|e| e.to_str());
        match ext {
            Some("toml") => {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    toml_stems.insert(stem.to_string());
                }
                candidates.push((path, true));
            }
            Some("theme") => candidates.push((path, false)),
            _ => {}
        }
    }

    // Migrate orphaned `.theme` files: parse + write `.toml` beside each, then
    // prefer the freshly written `.toml` (so serialization is canonical).
    let mut paths: Vec<std::path::PathBuf> = candidates
        .into_iter()
        .filter_map(|(path, is_toml)| {
            if is_toml {
                Some(path)
            } else {
                // `.theme` — only use if no `.toml` of the same stem exists.
                let stem = path.file_stem()?.to_str()?.to_string();
                if toml_stems.contains(&stem) {
                    None
                } else {
                    // Migrate now so subsequent loads read the `.toml` directly.
                    match migrate_legacy_theme(&path) {
                        Ok(toml_path) => Some(toml_path),
                        Err(err) => {
                            tracing::warn!(
                                legacy = %path.display(),
                                error = %err,
                                "skipping un-migratable legacy .theme"
                            );
                            None
                        }
                    }
                }
            }
        })
        .collect();
    paths.sort();

    let mut themes = Vec::new();
    for path in paths {
        themes.push(load_theme_definition(path)?);
    }
    Ok(themes)
}

/// Parses a `.theme` legacy file, writes the equivalent `.toml` next to it,
/// and returns the path to the new `.toml`. The legacy file is left in place.
fn migrate_legacy_theme(legacy_path: &Path) -> io::Result<std::path::PathBuf> {
    let theme = parse_legacy_theme_definition(&fs::read_to_string(legacy_path)?)?;
    let toml_path = legacy_path.with_extension("toml");
    save_theme_definition(&toml_path, &theme)?;
    tracing::info!(
        legacy = %legacy_path.display(),
        theme = %toml_path.display(),
        "migrated legacy .theme to TOML"
    );
    Ok(toml_path)
}

/// Parses the TOML theme format. Missing fields take their defaults.
pub fn parse_theme_definition(text: &str) -> io::Result<ThemeDefinition> {
    let file: ThemeFile = toml::from_str(text)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    let name = file.name.trim();
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "theme name is required",
        ));
    }
    Ok(file.into())
}

/// Renders a theme as TOML (the on-disk `.toml` format).
pub fn render_theme_definition(theme: &ThemeDefinition) -> String {
    toml::to_string_pretty(&ThemeFile::from(theme.clone())).expect("theme serializes to TOML")
}

/// Parses the retired `key=value` `.theme` format. Kept only as the migration
/// reader for pre-TOML installations.
pub fn parse_legacy_theme_definition(text: &str) -> io::Result<ThemeDefinition> {
    let mut name: Option<String> = None;
    let mut is_dark = false;
    let mut colors = default_colors();

    for (line_index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid theme line {}", line_index + 1),
            ));
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "name" => name = Some(value.to_string()),
            "is_dark" => is_dark = parse_preference_bool(value)?,
            "app_bg" => colors.app_bg = parse_hex_color(value)?,
            "panel_bg" => colors.panel_bg = parse_hex_color(value)?,
            "surface_bg" => colors.surface_bg = parse_hex_color(value)?,
            "text" => colors.text = parse_hex_color(value)?,
            "muted" => colors.muted = parse_hex_color(value)?,
            "border" => colors.border = parse_hex_color(value)?,
            "active_bg" => colors.active_bg = parse_hex_color(value)?,
            "active_text" => colors.active_text = parse_hex_color(value)?,
            _ => {}
        }
    }

    let name = name
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "theme name is required"))?;

    Ok(ThemeDefinition {
        name,
        is_dark,
        colors,
    })
}

fn parse_hex_color(value: &str) -> io::Result<u32> {
    let value = value.trim().trim_start_matches('#');
    if value.len() != 6 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid color '{value}'"),
        ));
    }
    u32::from_str_radix(value, 16).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}
