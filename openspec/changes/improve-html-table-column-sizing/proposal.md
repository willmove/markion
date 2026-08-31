# Proposal: improve-html-table-column-sizing

## Why

Word/datasheet-exported cover tables (empty `<th>`/`<td>` spacers plus a few content cells with `rowspan`/`colspan`) render poorly in Visual Edit, Split Preview, and Read mode: `html_table_grid_view` sizes every logical column with an equal `1fr` track, so fully-empty side columns each take an exact fraction of the table width (a 5-column cover table spends 40% of its width on two empty columns), and every `<th>` gets pipe-table header shading even when the entire header row is empty — producing a gray frame around squeezed content instead of the near-borderless cover layout browsers produce. Users report this as "renders badly" for real documents.

The root constraint is upstream: gpui 0.2.2's `grid_cols(count)` maps to taffy `repeat(count, minmax(0, 1fr))` and exposes no per-track sizing, so content-proportional columns cannot be expressed from application code.

## What Changes

- **Vendored gpui**: add `grid_col_weights(Vec<f32>)` to the `Styled` trait (`Style.grid_col_weights` → taffy `minmax(0, fr(w))` per weight), taking precedence over `grid_cols`. Mirrors the existing Markion gpui patch style; no behavior change for existing callers.
- **Grid model (`src/parse.rs`)**: add pure `html_table_column_weights(&HtmlTableGrid) -> Vec<f32>` approximating browser auto table layout — per column, the max over covering cells of (content-width score ÷ colspan), with CJK/full-width glyphs weighted 2, a nominal score for cell images, and a small floor so empty columns keep a sliver instead of collapsing under drawn borders. Add pure `html_table_row_has_visible_header(&[HtmlTableCell]) -> bool`. Fix the stale `has_rowspan` doc comment (no renderer uses a fixed row height; GPUI uses native `row_span`, DOCX uses `vMerge`).
- **Renderer (`src/app/preview.rs`)**: `html_table_grid_view` passes content weights as grid tracks (equal tracks remain the degenerate fallback), and applies header emphasis (semibold + shading) only when the cell's row contains at least one header cell with visible content, so all-empty header rows render as body cells.

Weights derive from the already-cached per-version `HtmlTableGrid` inside `PreviewBlock::Html`; no new derived-state surface, no recompute on keystroke.

### Non-goals

- Full CSS `table-layout: auto` (multi-pass min/max content measurement), `<colgroup>`/`<col width>` attributes, cell `align`/`valign`, or per-cell CSS classes.
- Removing borders from HTML tables (established Markion table style; browsers differ because the authored HTML carries no borders).
- Changing GFM pipe-table rendering, DOCX/PDF/LaTeX export, or Visual Edit HTML cell editing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `html-table-rendering`: raw HTML tables size columns from content instead of equal fractions, and header emphasis requires visible header content.

## Impact

- `vendor/zed/crates/gpui/src/{style.rs, styled.rs, taffy.rs}` — additive `grid_col_weights` API (~20 lines).
- `src/parse.rs` — two pure helpers + tests; doc-comment fix.
- `src/lib.rs`, `src/app/mod.rs` — re-export the helpers into the app module.
- `src/app/preview.rs` — `html_table_grid_view` consumes them.
- Invariants preserved: per-document-version derived caches (weights are computed from the cached grid at render time, or may be cached later without new state), `crates/*` stay GPUI-free (helpers live in the root crate's parse module).
