## Why

The built-in PNG/JPEG "snapshot" export rendered the raw text through an ASCII-only
8x8 bitmap font (`font8x8::BASIC_FONTS`). Every non-ASCII glyph — notably Chinese,
Japanese, and Korean — fell back to a hollow "missing glyph" box, so a document
with any CJK content exported as garbled tofu. The output also ignored Markdown
structure entirely: it was a single monospaced column of raw source text with no
headings, code blocks, tables, or images, so typography/排版 was poor.

## What Changes

- Replace the text-snapshot renderer with a real layout renderer that reuses the
  PDF writer's intermediate representation (`build_pdf_ir`) and its cosmic-text
  font pipeline (`crates/pdf`).
- PNG/JPEG snapshots now render real Markdown structure in a single continuous
  canvas: H1–H6 headings, paragraphs, bullet/ordered/task lists (nested),
  blockquotes, GFM alert callouts, fenced code blocks with a background and
  leftover-syntax coloring, tables with a bold header and column alignment,
  horizontal rules, and embedded local PNG/JPEG/SVG images scaled to the column.
- Text is shaped with the process-wide font system (per-OS CJK fonts first, then
  the bundled OFL Noto Sans SC subset, plus Libertinus Serif / DejaVu Sans Mono
  for Latin and code), so CJK glyphs render as real characters and no character is
  replaced by a placeholder. Headings/code/tables follow the same theme and font
  sizes as the built-in PDF writer, so the image matches the PDF typography.
- The document flows into one tall image (no pagination, no page-number footers);
  page size, margins, and the TOC/`[export.pdf]` options are otherwise honored for
  geometry. Removed the now-unreachable bitmap-font drawing helpers.

Non-goals: this change does not paginate the snapshot, add thumbnail/size options,
rasterize remote or data-URI images (those keep the existing text fallback), or
change PDF/DOCX/HTML/LaTeX/Markdown export fidelity.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `export`: PNG/JPEG export becomes a rich, font-complete layout snapshot matching
  the PDF writer's typography instead of a low-fidelity ASCII text dump.

## Impact

- Affected application code: `src/export.rs` (`write_image_export` replaces
  `write_image_snapshot`), `src/lib.rs` (Png/Jpeg export arms), `crates/pdf` (new
  `raster` module + `render_snapshot`/`DEFAULT_SCALE`), and `crates/pdf/Cargo.toml`
  (adds `image` and `resvg` for rasterizing the canvas and embedded SVG images).
- Rendering happens on the export path only; per-document preview caches and the
  on-screen editor are untouched. The heavy `image`/`resvg` deps are already in the
  workspace lockfile, so no new crate versions are pulled in.
- No persisted configuration, document contents, or Markdown parsing change.
