## 1. Layout IR

- [x] 1.1 Add `InlineImage` (payload, CSS-pixel size, `ascent_px`, alt) and `Run.inline_image` in `crates/pdf/src/ir.rs`; re-export from `lib.rs`.
- [x] 1.2 Heading outline titles use `alt` for inline-image runs instead of empty `text`.

## 2. Mixed paragraph layout

- [x] 2.1 In `shape_paragraph`, reserve an unbreakable no-break-space placeholder for each inline-image run, calibrate its advance to the SVG width in points, and record `ShapedInlineObject` (x, size, ascent, run index) instead of emitting the placeholder glyphs.
- [x] 2.2 Expand `ShapedLine` height / `baseline_offset` to the max of text and math metrics; scale an object that exceeds the wrap width down to fit.
- [x] 2.3 In `place_line_raw`, emit `PlacedItem::Svg` at `baseline - ascent`; SVG parse failure uses the run's `alt` as in-flow text. Apply the stored `baseline_offset` at every place site.

## 3. IR builder

- [x] 3.1 In `pdf_run`, render `span.math` through `MathRenderer::render_inline` into `Run.inline_image`; on failure keep the authored LaTeX as a code-styled run.

## 4. Tests

- [x] 4.1 Root-crate test: `$E = mc^2$` in prose becomes an SVG inline image on the IR, not a code-styled `$…$` run; invalid inline math stays authored source in code style.
- [x] 4.2 `markion-pdf` test: a paragraph with an inline path-only SVG produces a placed SVG atom on the same line as surrounding text and still wraps as an atom.
- [x] 4.3 `cargo test --workspace` stays green.
