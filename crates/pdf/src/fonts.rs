//! Font subsystem (design D3): one process-wide `fontdb` behind a
//! `OnceLock`, combining discovered system fonts with guaranteed bundled
//! fallbacks (Noto Sans SC subsets + Latin faces from typst-assets).
//! Bundled faces are registered after system fonts so per-OS CJK faces
//! (Microsoft YaHei, PingFang SC, Noto Sans CJK SC) win when present.

use std::sync::{Mutex, OnceLock};

use cosmic_text::FontSystem;
use cosmic_text::fontdb;

/// OFL-licensed Noto Sans SC subset (common-use Han + punctuation).
const NOTO_SC_REGULAR: &[u8] = include_bytes!("../fonts/NotoSansSC-Regular.otf");
const NOTO_SC_BOLD: &[u8] = include_bytes!("../fonts/NotoSansSC-Bold.otf");

/// Register the guaranteed bundled fallback faces into `db`.
fn load_bundled(db: &mut fontdb::Database) {
    db.load_font_data(NOTO_SC_REGULAR.to_vec());
    db.load_font_data(NOTO_SC_BOLD.to_vec());
    // typst-assets embeds Libertinus Serif (regular/bold/italic) and
    // DejaVu Sans Mono as `&'static [u8]` font data.
    for data in typst_assets::fonts() {
        db.load_font_data(data.to_vec());
    }
}

/// Bundled Latin serif used for PDF/snapshot body text. Named explicitly so
/// body shaping does not follow the host `serif` generic (fontconfig on
/// Linux can alias that to a Pi/Symbol face).
pub(crate) const BODY_SERIF_FAMILY: &str = "Libertinus Serif";

pub(crate) fn body_family() -> cosmic_text::Family<'static> {
    cosmic_text::Family::Name(BODY_SERIF_FAMILY)
}

/// Pi / Adobe-Symbol faces expose glyphs at Latin code points, so cosmic-text
/// treats them as covering English and never falls back. Drop them so a
/// missing Libertinus glyph cannot land on Standard Symbols L / OpenSymbol.
fn looks_like_pi_symbol_name(raw: &str) -> bool {
    let n: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    n == "symbol"
        || n == "symbolmt"
        || n.starts_with("standardsymbols")
        || n == "opensymbol"
        || n.contains("symbolsnerdfont")
        || n == "webdings"
        || n.starts_with("wingdings")
        || n.contains("zapfdingbats")
}

fn is_pi_or_adobe_symbol_face(face: &fontdb::FaceInfo) -> bool {
    std::iter::once(face.post_script_name.as_str())
        .chain(face.families.iter().map(|(n, _)| n.as_str()))
        .any(looks_like_pi_symbol_name)
}

fn strip_pi_symbol_faces(db: &mut fontdb::Database) {
    let ids: Vec<_> = db
        .faces()
        .filter(|face| is_pi_or_adobe_symbol_face(face))
        .map(|face| face.id)
        .collect();
    for id in ids {
        db.remove_face(id);
    }
}

/// Build a font database; `system` controls whether OS fonts are scanned.
fn build_db(system: bool) -> fontdb::Database {
    let mut db = fontdb::Database::new();
    if system {
        db.load_system_fonts();
    }
    load_bundled(&mut db);
    strip_pi_symbol_faces(&mut db);
    // fontdb takes fontconfig's last `serif` alias as Family::Serif. Pin the
    // generic to the bundled Latin serif so leftover `Family::Serif` call
    // sites cannot follow a Pi/Symbol alias. Headings and code keep host
    // generics.
    db.set_serif_family(BODY_SERIF_FAMILY);
    db
}

fn build_font_system(system: bool) -> FontSystem {
    FontSystem::new_with_locale_and_db("en-US".to_string(), build_db(system))
}

static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

/// Run `f` with the process-wide font system. The system scan happens once;
/// the bundled fallback faces are always registered.
pub fn with_font_system<R>(f: impl FnOnce(&mut FontSystem) -> R) -> R {
    let mutex = FONT_SYSTEM.get_or_init(|| Mutex::new(build_font_system(true)));
    // A poisoned mutex still holds a usable font system; keep exporting.
    let mut guard = mutex.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

/// A font system with only the bundled faces — no system fonts. Used by
/// tests to prove CJK text resolves to the bundled Noto faces even on a
/// font-poor system.
#[cfg(test)]
pub(crate) fn bundled_only_font_system() -> FontSystem {
    build_font_system(false)
}

#[cfg(test)]
mod tests {
    use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};

    use super::*;

    /// Chinese text shaped against a bundled-only font database must
    /// resolve to the bundled Noto Sans SC faces (no system CJK font).
    #[test]
    fn bundled_only_resolves_chinese_to_noto() {
        let mut fs = bundled_only_font_system();
        let mut buffer = Buffer::new(&mut fs, Metrics::new(12.0, 12.0 * 1.4));
        buffer.set_text(
            "你好，世界",
            &Attrs::new().family(Family::Serif),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut fs, true);

        let mut faces = Vec::new();
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let face = fs.db().face(glyph.font_id).expect("fontdb knows the face");
                faces.push(face.post_script_name.clone());
            }
        }
        assert!(!faces.is_empty(), "expected shaped glyphs");
        assert!(
            faces.iter().all(|name| name.contains("NotoSansSC")),
            "Han glyphs must come from the bundled Noto Sans SC subset, got {faces:?}"
        );
    }

    /// The bundled-only database also provides serif Latin (Libertinus) and
    /// monospace (DejaVu Sans Mono) fallbacks.
    #[test]
    fn bundled_only_has_latin_fallbacks() {
        let fs = bundled_only_font_system();
        let db = fs.db();
        let names: Vec<&str> = db.faces().map(|f| f.post_script_name.as_str()).collect();
        assert!(
            names.iter().any(|n| n.contains("LibertinusSerif")),
            "Libertinus Serif fallback missing: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.contains("DejaVuSansMono")),
            "DejaVu Sans Mono fallback missing: {names:?}"
        );
    }

    #[test]
    fn generic_serif_is_pinned_to_libertinus() {
        let bundled = bundled_only_font_system();
        assert_eq!(bundled.db().family_name(&Family::Serif), "Libertinus Serif");
        with_font_system(|fs| {
            assert_eq!(fs.db().family_name(&Family::Serif), "Libertinus Serif");
        });
    }

    fn shaped_postscript_names(fs: &mut FontSystem, text: &str, attrs: &Attrs) -> Vec<String> {
        let mut buffer = Buffer::new(fs, Metrics::new(12.0, 16.8));
        buffer.set_text(text, attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(fs, true);
        let mut faces = Vec::new();
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let face = fs.db().face(glyph.font_id).expect("fontdb knows the face");
                faces.push(face.post_script_name.clone());
            }
        }
        faces.sort();
        faces.dedup();
        faces
    }

    /// Regular and italic Latin body text must come from Libertinus Serif,
    /// never a Math or Symbol face that would draw Greek lookalikes.
    #[test]
    fn latin_body_shapes_with_libertinus_serif() {
        let mut fs = bundled_only_font_system();
        let regular = shaped_postscript_names(
            &mut fs,
            "This starter document is a quick tour of Markdown",
            &Attrs::new().family(Family::Serif),
        );
        assert!(
            !regular.is_empty() && regular.iter().all(|n| n.starts_with("LibertinusSerif")),
            "regular Latin must use Libertinus Serif, got {regular:?}"
        );

        let italic = shaped_postscript_names(
            &mut fs,
            "This starter document is a quick tour of Markdown",
            &Attrs::new()
                .family(Family::Serif)
                .style(cosmic_text::Style::Italic),
        );
        assert!(
            !italic.is_empty() && italic.iter().all(|n| n.starts_with("LibertinusSerif")),
            "italic Latin must use Libertinus Serif, got {italic:?}"
        );
        assert!(
            italic.iter().any(|n| n.contains("Italic")),
            "italic Latin should use the italic face, got {italic:?}"
        );
    }

    /// The process-wide database loads OS fonts first. Pinning + named body
    /// family must still win over Times New Roman / fontconfig `serif`.
    #[test]
    fn process_wide_latin_body_uses_libertinus_not_host_serif() {
        with_font_system(|fs| {
            let names = shaped_postscript_names(
                fs,
                "This starter document is a quick tour of Markdown",
                &Attrs::new().family(body_family()),
            );
            assert!(
                !names.is_empty() && names.iter().all(|n| n.starts_with("LibertinusSerif")),
                "export-path Latin body must use Libertinus Serif, got {names:?}"
            );
        });
    }

    #[test]
    fn adobe_symbol_family_names_are_detected() {
        assert!(looks_like_pi_symbol_name("Standard Symbols L"));
        assert!(looks_like_pi_symbol_name("StandardSymbolsL"));
        assert!(looks_like_pi_symbol_name("OpenSymbol"));
        assert!(looks_like_pi_symbol_name("Symbols Nerd Font"));
        assert!(looks_like_pi_symbol_name("Symbol"));
        assert!(!looks_like_pi_symbol_name("Noto Sans Symbols"));
        assert!(!looks_like_pi_symbol_name("Libertinus Serif"));
        assert!(!looks_like_pi_symbol_name("Segoe UI Symbol"));
        assert!(!looks_like_pi_symbol_name("Times New Roman"));
    }
}
