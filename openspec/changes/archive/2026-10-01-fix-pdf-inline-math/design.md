## Context

The built-in PDF writer (`crates/pdf` + `src/export.rs` `build_pdf_ir`) already embeds display math as a block-level `Image { Svg }` via `MathRenderer::render_block`. Inline math spans take a different path: `pdf_run` sees `InlineSpan.math` but the IR `Run` is text-only, so the authored `$…$` is emitted as a code-styled run. The original `improve-pdf-export` design (D4) already chose preview-SVG + `krilla-svg` for both inline and display math; only the inline half was deferred.

Paragraph layout is cosmic-text `Buffer` wrapping of styled runs. cosmic-text has no native inline-object API, so inline SVG must occupy a measured placeholder during wrapping and be swapped for a `PlacedItem::Svg` at emission. Export remains a one-shot read of the cached `preview_blocks()`; no typing-path work.

## Goals / Non-Goals

**Goals:**
- Typeset valid `$…$` (and any other inline math span) as a baseline-aligned SVG atom in prose, including headings, lists, table cells, and footnote bodies.
- Keep wrapping atomic: a formula does not split across lines; if it does not fit the remainder of the current line it moves whole to the next.
- On renderer or SVG-parse failure, keep the authored LaTeX in-flow as a code-styled run and still succeed.

**Non-Goals:**
- Changing display-math blocks, pandoc math, selectable math text, diagrams, or PDF options.

## Decisions

### D1 — Inline image on `Run`, not a new block kind

Add `Run.inline_image: Option<InlineImage>` with SVG (or raster) payload, CSS-pixel width/height, and `ascent_px` (top edge to formula baseline). Empty `text` on that run. Alternatives rejected: a `Run` enum (churn at every construction site) and promoting each formula to `Block::Image` (breaks in-flow layout).

The root crate fills this from `MathRenderer::render_inline(&math.latex)`; pixel metrics come from `RenderedMath`. Fallback on `Err` is a code-styled text run of `math.authored`.

### D2 — Unbreakable placeholder inside cosmic-text wrapping

For a run with `inline_image`, shape a zero-width-space plus a single Latin `M` whose advance is calibrated to the SVG width in points (`px * 72/96`). A letter is used instead of Unicode spaces because bundled faces can measure EM/NBSP as zero. Per-span `metrics_opt` sets the placeholder em size; after a first shape pass, scale that size by `desired/measured` and reshape so wrapping sees the true atom width. The leading ZWSP is a break opportunity before the atom so it does not glue to adjacent letters. Placeholder glyph groups are not emitted; their `start_x` becomes the SVG origin.

Rejected: hand-rolled mixed-fragment wrapping (would reimplement UAX#14/CJK breaking already owned by cosmic-text); treating math as its own paragraph (puts the formula on its own line).

A formula wider than the wrap width is scaled uniformly to fit (ascent included), matching the block-image “scale down to column” rule.

### D3 — Line metrics from max(text, math)

Each `ShapedLine` stores `baseline_offset` (currently `height * 0.8`). When the line carries inline objects, `ascent = max(text_ascent, math_ascent)` and `descent = max(text_descent, math_descent)`; `height` and `baseline_offset` update so the SVG sits on the text baseline (`y = baseline - ascent`) and tall constructs (fractions, superscripts) do not collide with the previous or next line.

### D4 — Parse SVG at place time

Reuse `usvg::Tree::from_str` + `PlacedItem::Svg` already used for block images. Parse failure degrades that run to the `alt` string (authored LaTeX) rather than failing the document.

## Risks / Trade-offs

- **[Placeholder width calibration misses a font]** → If the first-pass advance is 0, skip the placeholder and emit `alt` as code-styled text (same as renderer failure).
- **[Tall formula near a page break]** → `ensure_space` already page-breaks the whole line using the expanded `line.height`.
- **[Overlap if calibration is off]** → One reshape pass; tests assert the SVG is present and the following text starts after the object's width.

## Migration Plan

No config, file-format, or API migration. Rollback is revert. Pandoc PDF path is unchanged.

## Open Questions

None — renderer, SVG emission, and fallback contract already exist for display math.
