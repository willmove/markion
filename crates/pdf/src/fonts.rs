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

/// Build a font database; `system` controls whether OS fonts are scanned.
fn build_db(system: bool) -> fontdb::Database {
    let mut db = fontdb::Database::new();
    if system {
        db.load_system_fonts();
    }
    load_bundled(&mut db);
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
    use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};

    use super::*;

    /// Chinese text shaped against a bundled-only font database must
    /// resolve to the bundled Noto Sans SC faces (no system CJK font).
    #[test]
    fn bundled_only_resolves_chinese_to_noto() {
        let mut fs = bundled_only_font_system();
        let mut buffer = Buffer::new(&mut fs, Metrics::new(12.0, 12.0 * 1.4));
        buffer.set_text("你好，世界", &Attrs::new().family(Family::Serif), Shaping::Advanced, None);
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
}
