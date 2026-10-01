## Context

The built-in PDF/snapshot pipeline shapes text with cosmic-text against a process-wide `fontdb`. Body paragraphs request CSS generic `Family::Serif`. On Linux, `fontdb::Database::load_system_fonts` parses fontconfig aliases and **overwrites** `family_serif` with each alias's first `prefer` name; the last `serif` alias wins. That name can be a Pi/Symbol face (Standard Symbols L, OpenSymbol, a Nerd Font "Symbols" family, or a fontconfig leftover). Those faces expose glyphs at Latin code points using Adobe Symbol encoding, so `T` draws as `Τ`, `h` as `η`, `q` as `θ`. Bold/heading/code stacks miss that Regular face and fall back to a real Latin font — matching the Omarchy screenshot.

Bundled-only databases never load fontconfig, so `Family::Serif` stays `"Times New Roman"` (missing) and cosmic-text's fallback already reaches Libertinus Serif. The production path (`load_system_fonts` then bundled faces) does not.

Font setup is a process-wide `OnceLock`. It is not on the typing path and does not touch Markdown version caches.

```
Markdown IR  →  shape (Name("Libertinus Serif") / SansSerif / Monospace)
                      │
                      ▼
              fontdb generic name (pin serif → Libertinus)
                      │
         Linux: last fontconfig `serif` alias  ← ignored for body
                      │
              Pi/Symbol faces stripped from db
                      │
              CJK / missing glyphs still fallback
              (system CJK, then bundled Noto Sans SC)
```

## Goals / Non-Goals

**Goals:**

- Latin regular and italic body text in PDF (and PNG/JPEG snapshots sharing the same font system) keep authored Latin letters on Linux hosts whose fontconfig `serif` alias is a Symbol/Pi family.
- CJK fallback order stays: per-OS system CJK face, then bundled Noto Sans SC.
- One pin in `build_db` plus named body Attrs so leftover `Family::Serif` and the PDF/snapshot body path cannot follow fontconfig.

**Non-Goals:**

- Following the user's theme `rendered` font slot in PDF.
- Changing heading (`sans-serif`) or code (`monospace`) generic names.
- Removing unrelated system fonts from the database (CJK and coverage fallbacks still need them; only Pi/Adobe-Symbol families are dropped).
- Shipping a custom Fallback impl or extra crates.

## Decisions

1. **Pin `serif` to `"Libertinus Serif"` after load, and request that name for body Attrs.**  
   `raster.rs` previously used `Family::Serif` directly. The pin covers leftover generic call sites; body paragraphs and snapshot body text use `Family::Name("Libertinus Serif")` so they never consult the host generic even if the pin is skipped.

2. **Do not pin `sans-serif` / `monospace`.**  
   Headings and code already render Latin correctly on the failing host. Changing them would shift PDF typography (Arial/Liberation Sans → Noto Sans SC) without fixing the report.

3. **Drop Pi / Adobe-Symbol faces from fontdb by family/PostScript name.**  
   Cosmic-text will use a face that merely *claims* Latin coverage. Standard Symbols L / OpenSymbol claim `T` and draw `Τ`. Name-matching those families (not Unicode "Noto Sans Symbols" / "Segoe UI Symbol") removes them before shaping. No extra crate; no per-glyph cmap walk.

4. **Tests lock the pin, the export path, and the name filter.**  
   Assert `family_name(Serif) == "Libertinus Serif"` on bundled-only and process-wide databases. Shape regular and italic Latin on both databases and require Libertinus Serif PostScript names (this is red on Windows before the pin: Times New Roman). Render a one-paragraph PDF and require the Libertinus Serif name, not Symbol face names. Unit-test the Pi/Symbol name detector.

## Risks / Trade-offs

- **[Risk] Windows/macOS body text switches from Times New Roman (fontdb default) to Libertinus Serif.** → Acceptable: Libertinus is the already-bundled Latin fallback and matches Typst; PDFs become consistent across OS. Not a preview/editor change.
- **[Risk] A host with no working Libertinus load still falls through to other faces.** → Bundled `typst-assets` faces are registered in the same `build_db`; Pi/Symbol faces are stripped; existing tests require Libertinus on the process-wide path.
- **[Risk] Italic/bold still request weight/style on the pinned family.** → Libertinus Serif ships Regular, Italic, Bold, BoldItalic in `typst_assets::fonts()`.

## Migration Plan

No user-facing setting or file format change. Existing PDF exports regenerate with Libertinus body Latin on next export.

## Open Questions

None; the pin family name is the PostScript-adjacent family string already observed in the bundled faces (`"Libertinus Serif"`).
