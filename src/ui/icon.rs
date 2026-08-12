//! File-tree icon resources and rendering.
//!
//! Icons are compile-time-embedded SVGs (a Lucide subset, ISC licence — see
//! `assets/icons/ui/LICENSE.lucide`), drawn as a monochrome mask via GPUI's
//! `svg()` element. The colour comes from `text_color`, so icons follow the
//! active theme without needing a separate asset set per theme.
//!
//! Ported from termior's `termior-ui-kit/src/icon.rs` plus the explorer-side
//! `IconKind` mapping, collapsed into one module: markion keeps all UI in the
//! app crate and has no separate ui-kit crate.

use std::borrow::Cow;
use std::path::Path;

use gpui::{prelude::*, px, svg, AssetSource, Rgba, SharedString, Svg};

/// Declares the icon set: variant name → `assets/icons/ui/<file>.svg`.
///
/// Files are read at compile time; a typo'd filename is a compile error, not a
/// runtime missing-icon.
macro_rules! icon_set {
    ($($variant:ident => $file:literal),+ $(,)?) => {
        /// File-tree icons. Add an icon = one line in [`icon_set!`] + one SVG file.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        // An icon asset is "available" even if no caller renders it yet, so an
        // unused variant (e.g. plain `File`) is not a real liveness signal.
        #[allow(dead_code)]
        pub enum Icon {
            $($variant),+
        }

        impl Icon {
            /// Asset path resolved by [`IconAssets`] for GPUI's `svg().path(..)`.
            pub fn path(self) -> &'static str {
                match self {
                    $(Self::$variant => concat!("icons/ui/", $file, ".svg")),+
                }
            }

            /// Every variant, for test traversal.
            #[allow(dead_code)] // referenced only under cfg(test)
            pub const ALL: &'static [Icon] = &[$(Self::$variant),+];
        }

        const EMBEDDED: &[(&str, &[u8])] = &[
            $((
                concat!("icons/ui/", $file, ".svg"),
                include_bytes!(concat!("../../assets/icons/ui/", $file, ".svg")),
            )),+
        ];
    };
}

icon_set! {
    Folder => "folder",
    FolderOpen => "folder-open",
    File => "file",
    FileCode => "file-code",
    FileText => "file-text",
    FileImage => "file-image",
    FileCog => "file-cog",
    Braces => "braces",
}

/// GPUI asset source for embedded icons. Install once on `Application::with_assets`.
pub struct IconAssets;

impl AssetSource for IconAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(EMBEDDED
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(EMBEDDED
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect())
    }
}

/// File-tree icon edge length. Matches termior's `icon_size::SM` (16px) — the
/// size the row layout (root_view.rs) was already designed around.
const FILE_TREE_ICON_SIZE: f32 = 16.0;

/// Draws `icon` at the given edge length and colour.
pub fn icon(icon: Icon, size: f32, color: Rgba) -> Svg {
    svg()
        .path(icon.path())
        .size(px(size))
        .flex_none()
        .text_color(color)
}

/// Coarse file-type classification used to pick a tree icon.
///
/// Mirrors termior's explorer `IconKind`. Kept in lockstep so the two projects
/// classify the same extension the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Folder,
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    Html,
    Css,
    Json,
    Markdown,
    Image,
    Config,
    File,
}

/// Maps a path to its [`IconKind`] by name/extension. Direct port of termior's
/// `icon_for`, so the two projects stay in sync if either widens the table.
pub fn icon_for(path: &Path, is_dir: bool) -> IconKind {
    if is_dir {
        return IconKind::Folder;
    }
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    let ext = path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if matches!(name, "Cargo.toml" | "package.json" | "pyproject.toml") {
        return IconKind::Config;
    }
    match ext.as_str() {
        "rs" => IconKind::Rust,
        "js" | "jsx" | "mjs" | "cjs" => IconKind::JavaScript,
        "ts" | "tsx" | "mts" | "cts" => IconKind::TypeScript,
        "py" | "pyi" => IconKind::Python,
        "go" => IconKind::Go,
        "java" => IconKind::Java,
        "html" | "htm" => IconKind::Html,
        "css" | "scss" | "sass" | "less" => IconKind::Css,
        "json" | "jsonc" => IconKind::Json,
        "md" | "markdown" => IconKind::Markdown,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => IconKind::Image,
        "toml" | "yaml" | "yml" | "ini" => IconKind::Config,
        _ => IconKind::File,
    }
}

/// Picks the file-tree glyph + colour for a given [`IconKind`].
///
/// Folders use `folder_color` (the caller passes markion's `palette.active_text`
/// — its closest analogue to termior's accent); files use `file_color`, which
/// the caller has already derived from row state (active / selected / clickable
/// / muted). This function intentionally does NOT apply an additional alpha:
/// double-dimming would make file rows unreadable on muted rows.
pub fn file_tree_icon(
    kind: IconKind,
    expanded: bool,
    file_color: Rgba,
    folder_color: Rgba,
) -> Svg {
    let (glyph, tint) = match kind {
        IconKind::Folder if expanded => (Icon::FolderOpen, folder_color),
        IconKind::Folder => (Icon::Folder, folder_color),
        IconKind::Rust
        | IconKind::JavaScript
        | IconKind::TypeScript
        | IconKind::Python
        | IconKind::Go
        | IconKind::Java
        | IconKind::Html
        | IconKind::Css => (Icon::FileCode, file_color),
        IconKind::Json => (Icon::Braces, file_color),
        // Markdown keeps the "text document" glyph; plain-text files
        // (`.txt`/`.log`/`.csv`/…, which `icon_for` maps to `IconKind::File`)
        // get the plain file outline so the two are visually distinct.
        IconKind::Markdown => (Icon::FileText, file_color),
        IconKind::File => (Icon::File, file_color),
        IconKind::Image => (Icon::FileImage, file_color),
        IconKind::Config => (Icon::FileCog, file_color),
    };
    icon(glyph, FILE_TREE_ICON_SIZE, tint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_resolves_to_embedded_bytes() {
        for &variant in Icon::ALL {
            let bytes = IconAssets
                .load(variant.path())
                .expect("asset lookup succeeds")
                .unwrap_or_else(|| panic!("{variant:?} has no embedded bytes"));
            assert!(
                bytes.starts_with(b"<svg"),
                "{variant:?} does not look like an SVG"
            );
        }
    }

    #[test]
    fn unknown_paths_resolve_to_none() {
        assert!(IconAssets.load("icons/ui/nope.svg").unwrap().is_none());
    }

    #[test]
    fn listing_the_icon_directory_returns_every_icon() {
        let listed = IconAssets.list("icons/ui/").unwrap();
        assert_eq!(listed.len(), Icon::ALL.len());
    }

    #[test]
    fn icon_for_classifies_extensions_like_termior() {
        assert_eq!(icon_for(Path::new("src/main.rs"), false), IconKind::Rust);
        assert_eq!(icon_for(Path::new("README.md"), false), IconKind::Markdown);
        assert_eq!(icon_for(Path::new("pkg"), true), IconKind::Folder);
        assert_eq!(icon_for(Path::new("Cargo.toml"), false), IconKind::Config);
        assert_eq!(icon_for(Path::new("data.json"), false), IconKind::Json);
        assert_eq!(icon_for(Path::new("logo.png"), false), IconKind::Image);
        assert_eq!(icon_for(Path::new("unknown.xyz"), false), IconKind::File);
    }
}
