# Rich PNG/JPEG snapshot export

- [x] Add `crates/pdf` raster module that lays the layout IR into one continuous
      RGBA canvas using cosmic-text (headings, paragraphs, lists, quotes, alerts,
      code, tables, rules, images) and returns an `image::RgbaImage`.
- [x] Render text with the process-wide font system (system CJK fonts then the
      bundled Noto Sans SC subset; Latin via Libertinus Serif and DejaVu Sans Mono)
      so CJK glyphs are drawn as real characters with no tofu boxes.
- [x] Expose `markion_pdf::render_snapshot` / `DEFAULT_SCALE` from `crates/pdf`.
- [x] Add `image` (jpeg, png) and `resvg` 0.47 dependencies to `crates/pdf` so the
      canvas and embedded SVG images rasterize in-crate (both already in the
      lockfile).
- [x] Replace `src/export.rs` `write_image_snapshot` (ASCII bitmap) with
      `write_image_export`, which builds the PDF IR and rasterizes it; drop the
      bitmap-font helper functions.
- [x] Update `src/lib.rs` Png/Jpeg export arms to pass the document and export
      preferences.
- [x] Add `crates/pdf/src/raster.rs` tests: CJK text paints dark glyph pixels (not
      boxes), mixed CJK/Latin text wraps, empty documents still render a frame, and
      the public snapshot wrapper resolves CJK.
- [ ] Validate the change with `openspec validate rich-image-snapshot-export` and
      archive it so the `export` spec delta lands in `openspec/specs/export/`.
