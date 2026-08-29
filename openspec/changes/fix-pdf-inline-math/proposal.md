## Why

The built-in PDF writer already typesets display math through the preview's GPUI-free SVG renderer, but inline `$…$` formulas are dumped as code-styled LaTeX source. The export spec already requires both inline and display math to match preview; the IR simply has no inline-image container, so `pdf_run` never calls the renderer. Documents that mix prose and formulas therefore export with unreadable source instead of typeset atoms.

## What Changes

- Extend the `markion-pdf` layout IR so a styled run can carry an inline vector image (SVG, with pixel size and baseline ascent), not only text/link/footnote metadata.
- Lay those inline images out as atomic, baseline-aligned objects inside paragraph wrapping (headings, lists, table cells, and footnotes included), using the same `krilla-svg` embedding path as block math.
- Route inline math spans in `build_pdf_ir` through `MathRenderer::render_inline` (the same renderer preview and HTML export use). Renderer failure stays in-flow as a code-styled run of the byte-identical authored LaTeX; export still succeeds.
- Add an explicit inline-math scenario to the existing PDF math requirement so the gap cannot regress silently.
- **Non-goals**: pandoc/XeLaTeX math (already native LaTeX); changing display-math block behavior; selectable/copyable math text inside the PDF; diagram fences; new export options or UI.

This change does not touch per-version Markdown caches, memoized highlighting, or the typing path: math SVG is produced once per export from the cached `preview_blocks()`.

## Capabilities

### New Capabilities

- none

### Modified Capabilities

- `export`: the built-in PDF math requirement gains an inline-math scenario (typeset SVG atom in prose flow) and clarifies that unrenderable *inline* math falls back to a code-styled run rather than a block.

## Impact

- `crates/pdf` (`ir.rs`, `text.rs`, `layout.rs`, `lib.rs`): IR + mixed paragraph layout/emission. No new dependencies; still GPUI-free.
- `src/export.rs`: `pdf_run` / `pdf_runs` convert `InlineSpan.math` via `MathRenderer::render_inline`.
- Tests in the root crate (IR builder) and `markion-pdf` (inline object layout). `cargo test --workspace` must stay green.
