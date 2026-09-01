## Why

Raw HTML tables already parse into a correct `HtmlTableGrid` (5×7 for the reported Word cover sheet), but `html_table_grid_view` does not pin cells to those coordinates. GPUI's `col_span` / `row_span` helpers overwrite `col_start` / `row_start` with `Span..Span`, so Taffy auto-places every cell. Cover sheets that occupy side columns with `rowspan` then stack as full-width bands: `文档版本` and `01` appear as one run in **Read, Split Preview, and Visual Edit**. Empty spacer `<th>`/`<td>` still get 8px padding and pipe-table borders, so they paint as extra blank cards. Visual Edit wraps the same grid in a second rounded `overflow_hidden` chrome, which makes the collapse look worse but is not the cause — Read mode shows the same concatenation.

`improve-html-table-column-sizing` only reweights tracks. Weighted `fr` on auto-placed items cannot recover a column grid.

## What Changes

- **Renderer placement**: stop chaining `col_span` / `row_span` after `col_start` / `row_start`. Place each non-spacer cell with exclusive grid lines (`col_start` + `col_end`, `row_start` + `row_end`) so Taffy keeps the parser's `(column, row, colspan, rowspan)` footprint. Give the grid container a definite width (`w_full`) so weighted `fr` tracks resolve instead of collapsing under `overflow_hidden`.
- **Empty spacer cells**: cells with no visible text and no image SHALL occupy their tracks (so `rowspan` geometry stays intact) but SHALL NOT paint as padded bordered cards — no content padding and no internal grid stroke on those cells.
- **Visual Edit chrome**: when the HTML block's presentation is a table grid, do not nest a second rounded table-style border around it. Keep the hover `</>` source payload; the grid itself is the surface, matching Read mode.
- **Tests**: pin the reported cover-sheet HTML so `文档版本` and `01` land in different columns of the same row (grid model already does; add a render-placement helper or documented line-range assertion). Regression on a simple `rowspan="3"` datasheet so auto-placement cannot silently return.

Weights still come from the cached per-version `HtmlTableGrid` inside `PreviewBlock::Html`; no new derived-state surface, no keystroke reparse.

### Non-goals

- No change to HTML table parsing, `HtmlTableGrid`, export (DOCX/PDF/LaTeX), or GFM pipe tables.
- No Visual Edit cell editing, no `<colgroup>` / `width` / `align` / `valign`.
- No borderless HTML tables in general — only empty spacer cells drop padding and internal strokes; cells with content keep the pipe-table look.
- No scope merge into `improve-html-table-column-sizing` (column weights stay that change; this change makes those weights apply to the right tracks).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `html-table-rendering`: rendered HTML table cells occupy their resolved grid lines (not auto-placement); sibling cells in one row stay in separate columns; empty spacer cells do not paint as padded cards; Visual Edit shows that same grid without a second table chrome.

## Impact

- `src/app/preview.rs` — `html_table_grid_view` placement and empty-cell styling; `visual_html_editor` / `visual_collapsible_source_block` so a table presentation is not double-bordered.
- Optional tiny helper next to the renderer (or in `src/parse.rs` if kept GPUI-free) mapping `(col_start, colspan)` → exclusive end line for tests.
- Optional comment or thin wrapper around GPUI `col_span`/`row_span` documenting that they wipe start (no vendor API change required).
- Invariants preserved: per-document-version derived caches (`HtmlTableGrid` unchanged); `crates/*` stay GPUI-free; no file format or settings migration.
